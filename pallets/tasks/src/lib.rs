//! 任务版（Task Board）托盘模块
//!
//! 该模块实现了一个去中心化的任务管理系统，支持任务的创建、认领、提交、审批与拒绝。
//! 任务完成后，审批人可以向完成者发放资产（Point）作为奖励。

#![cfg_attr(not(feature = "std"), no_std)]   // 在 std 特性未启用时使用 no_std 环境

pub use pallet::*;  // 公开整个 pallet 模块以便外部使用

pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;


use crate::weights::WeightInfo;

use frame::prelude::*;   // 导入 FRAME 预定义基础类型与宏
use frame::traits::{
    tokens::fungibles::{Inspect, Mutate},  // 资产检查与修改 trait
    EnsureOrigin,                         // 用于校验调用来源
};

#[frame::pallet]  // 声明这是一个 pallet 模块
pub mod pallet {
    use super::*;   // 引入上层所有导入

    /// 任务 ID 类型
    pub type TaskId = u32;

    /// 资产 ID 类型，从 Config 中关联的 Assets 类型推导得到
    pub type AssetIdOf<T> =
        <<T as Config>::Assets as Inspect<
            <T as frame_system::Config>::AccountId,
        >>::AssetId;

    /// 余额类型，从 Config 中关联的 Assets 类型推导得到
    pub type BalanceOf<T> =
        <<T as Config>::Assets as Inspect<
            <T as frame_system::Config>::AccountId,
        >>::Balance;

    /// 任务状态枚举
    #[derive(
        Encode,
        Decode,
        Clone,
        Copy,
        PartialEq,
        Eq,
        RuntimeDebug,
        TypeInfo,
        MaxEncodedLen,
    )]
    pub enum TaskStatus {
        /// 初始状态：未认领，等待被接取
        Open,
        /// 已被某用户认领
        Claimed,
        /// 认领者已提交成果
        Submitted,
        /// 管理员已批准
        Approved,
        /// 管理员已拒绝
        Rejected,
    }

    /// 任务数据结构
    #[derive(
        Encode,
        Decode,
        Clone,
        PartialEq,
        Eq,
        RuntimeDebug,
        TypeInfo,
        MaxEncodedLen,
    )]
    pub struct Task<AccountId, Balance> {
        /// 任务创建者
        pub creator: AccountId,
        /// 任务接取者（认领后填写）
        pub assignee: Option<AccountId>,
        /// 完成后可获得的基础奖励
        pub reward: Balance,
        /// 当前任务状态
        pub status: TaskStatus,
    }

    /// Pallet 的配置 trait
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 运行时事件，需要包含本模块的事件
        type RuntimeEvent: From<Event<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 可操作的资产系统（需要支持 Inspect 和 Mutate）
        type Assets: Inspect<Self::AccountId> + Mutate<Self::AccountId>;

        /// 奖励发放时使用的具体资产 ID（提供者）
        type PointAssetId: Get<AssetIdOf<Self>>;

        /// 管理员权限校验（如审批、拒绝只能由管理员调用）
        type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        type WeightInfo: weights::WeightInfo;
    }

    /// Pallet 主体结构体
    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ------- 存储项 -------

    /// 自增任务 ID 计数器
    #[pallet::storage]
    #[pallet::getter(fn next_task_id)]   // 生成可读性好的 getter 方法
    pub type NextTaskId<T> = StorageValue<_, TaskId, ValueQuery>;

    /// 任务映射表，从 TaskId 到 Task 详情
    #[pallet::storage]
    #[pallet::getter(fn tasks)]
    pub type Tasks<T: Config> = StorageMap<
        _,
        Blake2_128Concat,                    // 使用哈希作为 key 的存储方案
        TaskId,
        Task<T::AccountId, BalanceOf<T>>,
        OptionQuery,                         // 可能为空
    >;

    // ------- 事件 -------

    /// 本模块抛出的事件
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)] // 自动生成入金事件的方法
    pub enum Event<T: Config> {
        /// 新任务被创建
        TaskCreated {
            task_id: TaskId,
            creator: T::AccountId,
            reward: BalanceOf<T>,
        },
        /// 任务被认领
        TaskClaimed {
            task_id: TaskId,
            assignee: T::AccountId,
        },
        /// 任务成果被提交
        TaskSubmitted {
            task_id: TaskId,
            assignee: T::AccountId,
        },
        /// 任务被管理员批准，同时发放奖励
        TaskApproved {
            task_id: TaskId,
            assignee: T::AccountId,
            reward: BalanceOf<T>,
        },
        /// 任务被管理员拒绝
        TaskRejected {
            task_id: TaskId,
        },
    }

    // ------- 错误 -------

    /// 本模块可能的错误类型
    #[pallet::error]
    pub enum Error<T> {
        /// 任务不存在
        TaskNotFound,
        /// 任务状态不是 Open（无法认领）
        TaskNotOpen,
        /// 任务状态不是 Claimed（无法提交）
        TaskNotClaimed,
        /// 任务状态不是 Submitted（无法审批）
        TaskNotSubmitted,
        /// 任务已被认领，不能重复认领
        AlreadyClaimed,
        /// 调用者不是该任务的认领者
        NotAssignee,
        /// 任务缺少认领者（内部异常）
        MissingAssignee,
        /// 任务 ID 溢出（超过 u32 最大值）
        TaskIdOverflow,
    }

    // ------- 公开调用函数 -------

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 创建一个新任务
        ///
        /// 参数: `reward` - 完成该任务后发放的奖励数额
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create_task())]
        pub fn create_task(
            origin: OriginFor<T>,
            reward: BalanceOf<T>,
        ) -> DispatchResult {
            // 确保调用者已签名，并获取其 AccountId
            let creator = ensure_signed(origin)?;

            // 获取下一个可用的任务 ID
            let task_id = NextTaskId::<T>::get();

            // 构造任务对象，初始状态为 Open，尚无认领者
            let task = Task {
                creator: creator.clone(),
                assignee: None,
                reward,
                status: TaskStatus::Open,
            };

            // 将任务存入存储
            Tasks::<T>::insert(task_id, task);

            // 任务 ID 自增，并防止溢出
            let next_id = task_id
                .checked_add(1)
                .ok_or(Error::<T>::TaskIdOverflow)?;
            NextTaskId::<T>::put(next_id);

            // 抛出事件
            Self::deposit_event(Event::TaskCreated {
                task_id,
                creator,
                reward,
            });

            Ok(())
        }

        /// 认领一个处于 Open 状态的任务
        ///
        /// 参数: `task_id` - 要认领的任务 ID
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::claim_task())]
        pub fn claim_task(
            origin: OriginFor<T>,
            task_id: TaskId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 尝试修改指定任务，闭包内进行状态检查
            Tasks::<T>::try_mutate(task_id, |maybe_task| -> DispatchResult {
                let task = maybe_task
                    .as_mut()
                    .ok_or(Error::<T>::TaskNotFound)?;

                // 只有 Open 状态的任务才能被认领
                ensure!(
                    task.status == TaskStatus::Open,
                    Error::<T>::TaskNotOpen
                );

                // 尚未有人认领（assignee 为 None）
                ensure!(
                    task.assignee.is_none(),
                    Error::<T>::AlreadyClaimed
                );

                // 绑定认领者并更新状态
                task.assignee = Some(who.clone());
                task.status = TaskStatus::Claimed;

                Ok(())
            })?;

            Self::deposit_event(Event::TaskClaimed {
                task_id,
                assignee: who,
            });

            Ok(())
        }

        /// 提交已完成的任务成果
        ///
        /// 仅允许认领者本人调用
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::submit_task())]
        pub fn submit_task(
            origin: OriginFor<T>,
            task_id: TaskId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Tasks::<T>::try_mutate(task_id, |maybe_task| -> DispatchResult {
                let task = maybe_task
                    .as_mut()
                    .ok_or(Error::<T>::TaskNotFound)?;

                // 只有 Claimed 状态的任务才能提交
                ensure!(
                    task.status == TaskStatus::Claimed,
                    Error::<T>::TaskNotClaimed
                );

                // 获取存有的认领者，不存在则报错
                let assignee = task
                    .assignee
                    .as_ref()
                    .ok_or(Error::<T>::MissingAssignee)?;

                // 只有认领者本人才能提交
                ensure!(assignee == &who, Error::<T>::NotAssignee);

                // 状态流转至 Submitted
                task.status = TaskStatus::Submitted;

                Ok(())
            })?;

            Self::deposit_event(Event::TaskSubmitted {
                task_id,
                assignee: who,
            });

            Ok(())
        }

        /// 管理员批准一个已提交的任务
        ///
        /// 批准后，会向认领者铸造对应奖励资产。
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::approve_task())]
        pub fn approve_task(
            origin: OriginFor<T>,
            task_id: TaskId,
        ) -> DispatchResult {
            // 仅允许管理员调用
            T::AdminOrigin::ensure_origin(origin)?;

            // 修改任务并取出认领者和奖励额
            let (assignee, reward) =
                Tasks::<T>::try_mutate(task_id, |maybe_task| {
                    let task = maybe_task
                        .as_mut()
                        .ok_or(Error::<T>::TaskNotFound)?;

                    // 只有 Submitted 状态的任务可以被审批
                    ensure!(
                        task.status == TaskStatus::Submitted,
                        Error::<T>::TaskNotSubmitted
                    );

                    let assignee = task
                        .assignee
                        .clone()
                        .ok_or(Error::<T>::MissingAssignee)?;

                    let reward = task.reward;

                    // 将状态置为 Approved
                    task.status = TaskStatus::Approved;

                    Ok::<_, DispatchError>((assignee, reward))
                })?;

            // 向认领者铸造奖励资产
            T::Assets::mint_into(
                T::PointAssetId::get(),
                &assignee,
                reward,
            )?;

            Self::deposit_event(Event::TaskApproved {
                task_id,
                assignee,
                reward,
            });

            Ok(())
        }

        /// 管理员拒绝一个已提交的任务
        ///
        /// 任务状态变为 Rejected，不发放奖励。
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::reject_task())]
        pub fn reject_task(
            origin: OriginFor<T>,
            task_id: TaskId,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            Tasks::<T>::try_mutate(task_id, |maybe_task| -> DispatchResult {
                let task = maybe_task
                    .as_mut()
                    .ok_or(Error::<T>::TaskNotFound)?;

                // 同样要求状态为 Submitted
                ensure!(
                    task.status == TaskStatus::Submitted,
                    Error::<T>::TaskNotSubmitted
                );

                task.status = TaskStatus::Rejected;

                Ok(())
            })?;

            Self::deposit_event(Event::TaskRejected { task_id });

            Ok(())
        }
    }
}