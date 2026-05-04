# Substrate / FRAME 入门指南与示例

这份文档用于帮助读者理解本仓库里的 Substrate / Polkadot SDK 项目结构。它先解释 Node、Runtime、FRAME、Pallet 的关系，再用 Counter 和 Task Rewards 两个示例说明一个 pallet 从开发、测试、benchmark、weight 到 storage migration / runtime upgrade 的完整链路。

如果你是第一次看这个仓库，建议阅读顺序如下：

1. 先阅读本文件，理解 Substrate / FRAME 的基本开发模型。
2. 再阅读 `docs/Substrate 企业任务平台.md`，了解本仓库的企业任务平台业务实现。
3. 在 WSL 中执行 `cargo test -p pallet-tasks` 和 `cargo check -p parachain-template-runtime`，确认当前代码能通过基础验证。
4. 用 Polkadot.js Apps 连接本地链，按企业任务平台文档里的账号和 origin 流程手动跑通任务、积分、委员会、scheduler 和 NFT 证书。

文档中的路径都按仓库相对路径描述，例如 `pallets/tasks/src/lib.rs`、`runtime/src/configs/mod.rs`。命令默认在 WSL 中执行。

## 1. Substrate 是什么

Substrate 是 Polkadot 生态里的区块链开发框架，现在属于 Polkadot SDK 的核心部分。

可以这样理解：

```text
Substrate = 用 Rust 开发区块链的底层框架
FRAME     = 编写链上 runtime 业务逻辑的模块化框架
Pallet    = FRAME 里的业务模块，例如余额、资产、治理、NFT、自定义任务模块
Runtime   = 链的规则层，决定交易怎么执行、状态怎么变化
Node      = 节点层，负责网络、交易池、同步、RPC、数据库、共识接入
```

Substrate 适合的场景：

- 想开发一条定制链。
- 想改账户、资产、手续费、治理等链级规则。
- 想把业务规则直接写进 runtime。
- 想练 Polkadot / AppChain / Parachain 方向。

如果只是写普通 DeFi、NFT、DApp，智能合约平台更快；如果要定制链规则，Substrate 更合适。

## 2. 核心组成

### 2.1 Node

节点层负责链运行所需的基础设施：

- P2P 网络通信。
- 区块同步。
- 交易池。
- RPC 接口。
- 共识执行。
- 数据库存储。

初学阶段尽量少改 `node/`。

### 2.2 Runtime

Runtime 是链的状态转换逻辑。

它决定：

- 哪些交易合法。
- 账户余额如何变化。
- 资产如何发行。
- 手续费如何计算。
- 链上业务状态如何更新。

Runtime 会编译成 Wasm，支持链上无分叉升级。

### 2.3 FRAME / Pallet

FRAME 是写 runtime 业务模块的框架。

一个典型 pallet 包含：

```text
Config   配置接口
Storage  链上存储
Call     可调用交易 extrinsic
Event    成功事件
Error    失败错误
Hooks    生命周期钩子
Tests    单元测试
Benchmark 权重基准测试
Migration 存储迁移
```

## 3. 开发环境

推荐环境：

```text
Ubuntu / WSL2
Rust 1.88
VS Code + rust-analyzer
Polkadot.js Apps
Polkadot SDK Parachain Template
```

Windows 建议使用 WSL2。

安装依赖：

```bash
sudo apt update

sudo apt install --assume-yes \
  git clang curl libssl-dev llvm libclang-dev \
  libudev-dev make protobuf-compiler
```

安装 Rust：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

配置工具链：

```bash
rustup toolchain install 1.88.0
rustup default 1.88.0
rustup target add wasm32-unknown-unknown --toolchain 1.88.0
rustup component add rust-src --toolchain 1.88.0
```

## 4. 本仓库启动与模板项目结构

如果你是从 GitHub 克隆本仓库，进入项目目录：

```bash
cd parachain-template
```

典型目录：

```text
parachain-template/
├── node/
├── pallets/
├── runtime/
├── Cargo.toml
└── README.md
```

常用检查命令：

```bash
cargo test -p pallet-tasks
cargo check -p parachain-template-runtime
```

完整 release 编译：

```bash
cargo build --release --locked
```

如果需要从原始 Polkadot SDK parachain template 初始化一个全新项目，可以参考：

```bash
git clone https://github.com/paritytech/polkadot-sdk-parachain-template.git parachain-template
cd parachain-template
```

本仓库已经基于模板完成了企业任务平台相关改造，日常学习和验证应优先使用当前仓库代码。

安装本地运行工具：

```bash
cargo install --locked staging-chain-spec-builder@16.0.0
cargo install --locked polkadot-omni-node@0.13.2
```

生成开发链配置：

```bash
chain-spec-builder create -t development \
  --relay-chain paseo \
  --para-id 1000 \
  --runtime ./target/release/wbuild/parachain-template-runtime/parachain_template_runtime.compact.compressed.wasm \
  named-preset development
```

启动本地链：

```bash
polkadot-omni-node --chain ./chain_spec.json --dev
```

Polkadot.js Apps 连接：

```text
ws://localhost:9944
```

常用页面：

- `Developer -> Extrinsics`
- `Developer -> Chain state`
- `Developer -> Events`

## 5. FRAME 开发基本模式

Pallet 业务逻辑通常遵循：

```text
读取链上状态
  -> 校验 origin
  -> 校验业务条件
  -> 修改 storage
  -> 发 event
  -> 返回 DispatchResult
```

最小结构：

```rust
#[pallet::config]
pub trait Config: frame_system::Config {
    type RuntimeEvent: From<Event<Self>>
        + IsType<<Self as frame_system::Config>::RuntimeEvent>;
}

#[pallet::storage]
pub type SomeStorage<T> = StorageValue<_, u32, ValueQuery>;

#[pallet::event]
pub enum Event<T: Config> {}

#[pallet::error]
pub enum Error<T> {}

#[pallet::call]
impl<T: Config> Pallet<T> {
    pub fn some_call(origin: OriginFor<T>) -> DispatchResult {
        let who = ensure_signed(origin)?;
        Ok(())
    }
}
```

## 6. 示例一：Counter Pallet

目标：链上保存一个 `Counter`，用户调用 `increment()` 后加 1。

### 6.1 目录

```text
pallets/counter/
├── Cargo.toml
└── src/
    └── lib.rs
```

### 6.2 `pallets/counter/Cargo.toml`

```toml
[package]
name = "pallet-counter"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
codec = { package = "parity-scale-codec", workspace = true, default-features = false, features = ["derive"] }
scale-info = { workspace = true, default-features = false, features = ["derive"] }
frame-support = { workspace = true, default-features = false }
frame-system = { workspace = true, default-features = false }

[features]
default = ["std"]
std = [
    "codec/std",
    "scale-info/std",
    "frame-support/std",
    "frame-system/std",
]
```

### 6.3 `pallets/counter/src/lib.rs`

```rust
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::storage]
    #[pallet::getter(fn counter)]
    pub type Counter<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Incremented {
            who: T::AccountId,
            value: u32,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        Overflow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn increment(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let new_value = Counter::<T>::get()
                .checked_add(1)
                .ok_or(Error::<T>::Overflow)?;

            Counter::<T>::put(new_value);

            Self::deposit_event(Event::Incremented {
                who,
                value: new_value,
            });

            Ok(())
        }
    }
}
```

### 6.4 Counter 接入 runtime

根 `Cargo.toml` workspace 增加：

```toml
members = [
    "node",
    "runtime",
    "pallets/counter",
]

[workspace.dependencies]
pallet-counter = { path = "pallets/counter", default-features = false }
```

`runtime/Cargo.toml` 增加：

```toml
pallet-counter = { workspace = true }
```

`std` feature 增加：

```toml
"pallet-counter/std",
```

`runtime/src/lib.rs`：

```rust
impl pallet_counter::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
}
```

注册到 runtime：

```rust
Counter: pallet_counter,
```

### 6.5 Counter 测试流程

编译：

```bash
cargo check
cargo build --release --locked
```

重新生成 chain spec 并启动：

```bash
chain-spec-builder create -t development \
  --relay-chain paseo \
  --para-id 1000 \
  --runtime ./target/release/wbuild/parachain-template-runtime/parachain_template_runtime.compact.compressed.wasm \
  named-preset development

polkadot-omni-node --chain ./chain_spec.json --dev
```

Polkadot.js Apps：

```text
Developer -> Extrinsics -> counter.increment()
Developer -> Chain state -> counter.counter()
Developer -> Events -> counter.Incremented
```

预期：

```text
初始 Counter = 0
Alice 调用 increment -> Counter = 1
Bob 调用 increment -> Counter = 2
```

## 7. Counter 单元测试结构

如果要给 Counter 补单元测试，常见结构：

```text
pallets/counter/src/mock.rs
pallets/counter/src/tests.rs
```

`lib.rs` 底部引入：

```rust
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
```

### 7.1 `mock.rs` 示例

```rust
use crate as pallet_counter;
use frame_support::{construct_runtime, parameter_types};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
    pub enum Test {
        System: frame_system,
        Counter: pallet_counter,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type RuntimeBlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = sp_runtime::testing::Header;
    type RuntimeBlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type Block = Block;
}

impl pallet_counter::Config for Test {
    type RuntimeEvent = RuntimeEvent;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    storage.into()
}
```

> 说明：不同 Polkadot SDK 版本的 `frame_system::Config` 类型项可能不同。如果编译报类型项不匹配，以模板已有 mock 为准调整。

### 7.2 `tests.rs` 示例

```rust
use crate::{mock::*, Counter as CounterStorage};
use frame_support::{assert_noop, assert_ok};

#[test]
fn increment_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(CounterStorage::<Test>::get(), 0);

        assert_ok!(Counter::increment(RuntimeOrigin::signed(1)));
        assert_eq!(CounterStorage::<Test>::get(), 1);

        assert_ok!(Counter::increment(RuntimeOrigin::signed(2)));
        assert_eq!(CounterStorage::<Test>::get(), 2);
    });
}
```

运行：

```bash
cargo test -p pallet-counter
```

## 8. 示例二：Task Rewards Pallet

Counter 只适合理解基本结构。更接近业务的练习是 `task-rewards`：

目标：

- 创建任务。
- 用户提交任务。
- 创建者审核提交。
- 审核通过后奖励用户。
- 支持押金、截止时间、最大提交数。
- 后续做 storage migration。

## 9. Task Rewards 数据结构

```rust
pub type TaskId = u32;

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum TaskStatus {
    Open,
    Closed,
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum SubmissionStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct TaskInfo<AccountId, Balance, BlockNumber> {
    pub creator: AccountId,
    pub reward: u32,
    pub deposit: Balance,
    pub deadline: BlockNumber,
    pub status: TaskStatus,
    pub max_submissions: u32,
    pub submission_count: u32,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct SubmissionInfo<AccountId, BlockNumber> {
    pub submitter: AccountId,
    pub submitted_at: BlockNumber,
    pub status: SubmissionStatus,
}
```

## 10. Task Rewards Storage

```rust
#[pallet::storage]
#[pallet::getter(fn next_task_id)]
pub type NextTaskId<T> = StorageValue<_, TaskId, ValueQuery>;

#[pallet::storage]
#[pallet::getter(fn tasks)]
pub type Tasks<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    TaskId,
    TaskInfo<T::AccountId, BalanceOf<T>, BlockNumberFor<T>>,
    OptionQuery
>;

#[pallet::storage]
#[pallet::getter(fn submissions)]
pub type Submissions<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    TaskId,
    Blake2_128Concat,
    T::AccountId,
    SubmissionInfo<T::AccountId, BlockNumberFor<T>>,
    OptionQuery
>;
```

## 11. Task Rewards Config

```rust
type BalanceOf<T> = <<T as Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId
>>::Balance;

#[pallet::config]
pub trait Config: frame_system::Config {
    type RuntimeEvent: From<Event<Self>>
        + IsType<<Self as frame_system::Config>::RuntimeEvent>;

    #[pallet::constant]
    type TaskDeposit: Get<BalanceOf<Self>>;

    #[pallet::constant]
    type DefaultMaxSubmissions: Get<u32>;

    type Currency: ReservableCurrency<Self::AccountId>;

    type WeightInfo: WeightInfo;
}
```

## 12. Task Rewards Event / Error

```rust
#[pallet::event]
#[pallet::generate_deposit(pub(super) fn deposit_event)]
pub enum Event<T: Config> {
    TaskCreated {
        task_id: TaskId,
        creator: T::AccountId,
        reward: u32,
        deadline: BlockNumberFor<T>,
        max_submissions: u32,
    },
    TaskSubmitted {
        task_id: TaskId,
        submitter: T::AccountId,
    },
    SubmissionApproved {
        task_id: TaskId,
        submitter: T::AccountId,
        reward: u32,
    },
}

#[pallet::error]
pub enum Error<T> {
    InvalidReward,
    InvalidDeadline,
    InvalidMaxSubmissions,
    TaskIdOverflow,
    TaskNotFound,
    TaskNotOpen,
    TaskExpired,
    AlreadySubmitted,
    MaxSubmissionsReached,
    SubmissionNotFound,
    NotTaskCreator,
    SubmissionNotPending,
}
```

## 13. Task Rewards Call 核心代码

### 13.1 `create_task`

```rust
#[pallet::call_index(0)]
#[pallet::weight(T::WeightInfo::create_task())]
pub fn create_task(
    origin: OriginFor<T>,
    reward: u32,
    deadline: BlockNumberFor<T>,
    max_submissions: u32,
) -> DispatchResult {
    let creator = ensure_signed(origin)?;

    ensure!(reward > 0, Error::<T>::InvalidReward);
    ensure!(max_submissions > 0, Error::<T>::InvalidMaxSubmissions);

    let current_block = frame_system::Pallet::<T>::block_number();
    ensure!(deadline > current_block, Error::<T>::InvalidDeadline);

    let task_id = NextTaskId::<T>::get();
    let next_task_id = task_id.checked_add(1).ok_or(Error::<T>::TaskIdOverflow)?;

    let deposit = T::TaskDeposit::get();
    T::Currency::reserve(&creator, deposit)?;

    let task = TaskInfo {
        creator: creator.clone(),
        reward,
        deposit,
        deadline,
        status: TaskStatus::Open,
        max_submissions,
        submission_count: 0,
    };

    Tasks::<T>::insert(task_id, task);
    NextTaskId::<T>::put(next_task_id);

    Self::deposit_event(Event::TaskCreated {
        task_id,
        creator,
        reward,
        deadline,
        max_submissions,
    });

    Ok(())
}
```

### 13.2 `submit_task`

```rust
#[pallet::call_index(1)]
#[pallet::weight(T::WeightInfo::submit_task())]
pub fn submit_task(origin: OriginFor<T>, task_id: TaskId) -> DispatchResult {
    let submitter = ensure_signed(origin)?;
    let current_block = frame_system::Pallet::<T>::block_number();

    Tasks::<T>::try_mutate(task_id, |maybe_task| -> DispatchResult {
        let task = maybe_task.as_mut().ok_or(Error::<T>::TaskNotFound)?;

        ensure!(task.status == TaskStatus::Open, Error::<T>::TaskNotOpen);
        ensure!(task.deadline >= current_block, Error::<T>::TaskExpired);
        ensure!(task.submission_count < task.max_submissions, Error::<T>::MaxSubmissionsReached);
        ensure!(
            !Submissions::<T>::contains_key(task_id, &submitter),
            Error::<T>::AlreadySubmitted
        );

        let submission = SubmissionInfo {
            submitter: submitter.clone(),
            submitted_at: current_block,
            status: SubmissionStatus::Pending,
        };

        Submissions::<T>::insert(task_id, &submitter, submission);
        task.submission_count = task.submission_count.saturating_add(1);

        Ok(())
    })?;

    Self::deposit_event(Event::TaskSubmitted { task_id, submitter });

    Ok(())
}
```

### 13.3 `approve_submission`

```rust
#[pallet::call_index(2)]
#[pallet::weight(T::WeightInfo::approve_submission())]
pub fn approve_submission(
    origin: OriginFor<T>,
    task_id: TaskId,
    submitter: T::AccountId,
) -> DispatchResult {
    let who = ensure_signed(origin)?;

    let task = Tasks::<T>::get(task_id).ok_or(Error::<T>::TaskNotFound)?;
    ensure!(task.creator == who, Error::<T>::NotTaskCreator);

    Submissions::<T>::try_mutate(task_id, &submitter, |maybe_submission| -> DispatchResult {
        let submission = maybe_submission
            .as_mut()
            .ok_or(Error::<T>::SubmissionNotFound)?;

        ensure!(
            submission.status == SubmissionStatus::Pending,
            Error::<T>::SubmissionNotPending
        );

        submission.status = SubmissionStatus::Approved;

        Ok(())
    })?;

    T::Currency::transfer(
        &task.creator,
        &submitter,
        task.reward.into(),
        ExistenceRequirement::KeepAlive,
    )?;

    Self::deposit_event(Event::SubmissionApproved {
        task_id,
        submitter,
        reward: task.reward,
    });

    Ok(())
}
```

> 注意：`task.reward.into()` 是否能编译取决于 `Balance` 类型。更稳的做法是把 `reward` 也定义成 `BalanceOf<T>`，但初学时用 `u32` 更直观，遇到类型报错就统一改为 `BalanceOf<T>`。

## 14. Task Rewards 测试 mock

测试需要 mock runtime，至少包含：

- `System`
- `Balances`
- `TaskRewards`

关键配置：

```rust
parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
    pub const TaskDeposit: u64 = 10;
    pub const DefaultMaxSubmissions: u32 = 100;
}

impl pallet_task_rewards::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type TaskDeposit = TaskDeposit;
    type DefaultMaxSubmissions = DefaultMaxSubmissions;
    type Currency = Balances;
    type WeightInfo = ();
}
```

Genesis 初始化账户余额：

```rust
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 1000), (2, 1000), (3, 1000)],
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    storage.into()
}
```

## 15. Task Rewards 单元测试

### 15.1 创建任务成功

```rust
#[test]
fn create_task_works_and_reserves_deposit() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        assert_ok!(TaskRewards::create_task(
            RuntimeOrigin::signed(1),
            10,
            100,
            2
        ));

        let task = Tasks::<Test>::get(0).expect("task should exist");

        assert_eq!(task.creator, 1);
        assert_eq!(task.reward, 10);
        assert_eq!(task.deposit, 10);
        assert_eq!(task.deadline, 100);
        assert_eq!(task.status, TaskStatus::Open);
        assert_eq!(task.max_submissions, 2);
        assert_eq!(task.submission_count, 0);
        assert_eq!(NextTaskId::<Test>::get(), 1);
    });
}
```

### 15.2 提交任务成功

```rust
#[test]
fn submit_task_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        assert_ok!(TaskRewards::create_task(RuntimeOrigin::signed(1), 10, 100, 2));
        assert_ok!(TaskRewards::submit_task(RuntimeOrigin::signed(2), 0));

        let submission = Submissions::<Test>::get(0, 2).expect("submission exists");
        assert_eq!(submission.submitter, 2);
        assert_eq!(submission.submitted_at, 1);
        assert_eq!(submission.status, SubmissionStatus::Pending);

        let task = Tasks::<Test>::get(0).expect("task exists");
        assert_eq!(task.submission_count, 1);
    });
}
```

### 15.3 达到最大提交数后拒绝

```rust
#[test]
fn submit_task_rejects_when_max_submissions_reached() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        assert_ok!(TaskRewards::create_task(RuntimeOrigin::signed(1), 10, 100, 1));
        assert_ok!(TaskRewards::submit_task(RuntimeOrigin::signed(2), 0));

        assert_noop!(
            TaskRewards::submit_task(RuntimeOrigin::signed(3), 0),
            Error::<Test>::MaxSubmissionsReached
        );

        let task = Tasks::<Test>::get(0).expect("task exists");
        assert_eq!(task.submission_count, 1);
    });
}
```

### 15.4 审核通过并发奖励

```rust
#[test]
fn approve_submission_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        assert_ok!(TaskRewards::create_task(RuntimeOrigin::signed(1), 10, 100, 2));
        assert_ok!(TaskRewards::submit_task(RuntimeOrigin::signed(2), 0));

        let before = Balances::free_balance(2);

        assert_ok!(TaskRewards::approve_submission(
            RuntimeOrigin::signed(1),
            0,
            2
        ));

        let submission = Submissions::<Test>::get(0, 2).expect("submission exists");
        assert_eq!(submission.status, SubmissionStatus::Approved);
        assert_eq!(Balances::free_balance(2), before + 10);
    });
}
```

运行：

```bash
cargo test -p pallet-task-rewards --lib
```

## 16. Benchmark 与 Weight 流程

Substrate 交易需要 weight，表示执行成本。

开发流程：

```text
先用固定 weight 跑通功能
  -> 写 benchmarking.rs
  -> 运行 benchmark
  -> 生成 weights.rs
  -> runtime 使用 WeightInfo
```

### 16.1 `WeightInfo` trait 示例

```rust
pub trait WeightInfo {
    fn create_task() -> Weight;
    fn submit_task() -> Weight;
    fn approve_submission() -> Weight;
}

impl WeightInfo for () {
    fn create_task() -> Weight {
        Weight::from_parts(10_000, 0)
    }

    fn submit_task() -> Weight {
        Weight::from_parts(10_000, 0)
    }

    fn approve_submission() -> Weight {
        Weight::from_parts(10_000, 0)
    }
}
```

### 16.2 Benchmark 要点

如果 `create_task` 增加了参数：

```rust
create_task(origin, reward, deadline, max_submissions)
```

那么 benchmark 和测试都要同步加参数。

示例：

```rust
#[benchmark]
fn create_task() {
    let caller: T::AccountId = whitelisted_caller();
    fund_account::<T>(&caller);

    let reward: u32 = 10;
    let max_submissions: u32 = 100;

    let current_block: BlockNumberFor<T> = 1u32.into();
    frame_system::Pallet::<T>::set_block_number(current_block);
    let deadline = current_block + 100u32.into();

    #[extrinsic_call]
    create_task(
        RawOrigin::Signed(caller.clone()),
        reward,
        deadline,
        max_submissions
    );

    let task = Tasks::<T>::get(0).expect("task should exist");
    assert_eq!(task.reward, reward);
    assert_eq!(task.max_submissions, max_submissions);
    assert_eq!(task.submission_count, 0);
}
```

运行示例：

```bash
cargo test -p pallet-task-rewards --features runtime-benchmarks --lib
```

## 17. Storage Migration：V1 到 V2

当 storage 结构变化时，必须考虑旧链数据迁移。

V1：

```rust
TaskInfo {
    creator,
    reward,
    deposit,
    deadline,
    status,
}
```

V2：

```rust
TaskInfo {
    creator,
    reward,
    deposit,
    deadline,
    status,
    max_submissions,
    submission_count,
}
```

要做：

```text
定义旧结构 OldTaskInfo
  -> on_runtime_upgrade 中 translate
  -> 给新字段补默认值
  -> 更新 StorageVersion
  -> 写 migration test
```

## 18. Storage Version

```rust
const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

#[pallet::pallet]
#[pallet::storage_version(STORAGE_VERSION)]
pub struct Pallet<T>(_);
```

## 19. OldTaskInfo

字段顺序必须和 V1 完全一致。

```rust
#[derive(
    Encode,
    Decode,
    Clone,
    Eq,
    PartialEq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
pub struct OldTaskInfo<AccountId, Balance, BlockNumber> {
    pub creator: AccountId,
    pub reward: u32,
    pub deposit: Balance,
    pub deadline: BlockNumber,
    pub status: TaskStatus,
}
```

## 20. Migration Hook

```rust
#[pallet::hooks]
impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    fn on_runtime_upgrade() -> Weight {
        let onchain_version = Pallet::<T>::on_chain_storage_version();

        if onchain_version == StorageVersion::new(1) {
            let mut migrated: u64 = 0;

            Tasks::<T>::translate::<OldTaskInfo<
                <T as frame_system::Config>::AccountId,
                BalanceOf<T>,
                BlockNumberFor<T>,
            >, _>(|_task_id, old_task| {
                migrated = migrated.saturating_add(1);

                Some(TaskInfo {
                    creator: old_task.creator,
                    reward: old_task.reward,
                    deposit: old_task.deposit,
                    deadline: old_task.deadline,
                    status: old_task.status,
                    max_submissions: T::DefaultMaxSubmissions::get(),
                    submission_count: 0,
                })
            });

            STORAGE_VERSION.put::<Pallet<T>>();

            T::DbWeight::get().reads_writes(migrated + 1, migrated + 1)
        } else {
            T::DbWeight::get().reads(1)
        }
    }
}
```

## 21. Migration Test

关键点：

- 写入旧结构时类型必须明确。
- 不要 `unhashed::put(&key, &old_task.encode())`，这会二次编码。
- 使用 `unhashed::put(&key, &old_task)`。
- `OldTaskInfo` 字段顺序必须和 V1 一致。

示例：

```rust
#[test]
fn migration_v1_to_v2_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let old_task: crate::OldTaskInfo<u64, u64, BlockNumberFor<Test>> =
            crate::OldTaskInfo {
                creator: 1u64,
                reward: 10u32,
                deposit: 10u64,
                deadline: 100u64,
                status: TaskStatus::Open,
            };

        let key = Tasks::<Test>::hashed_key_for(0u32);
        unhashed::put(&key, &old_task);

        StorageVersion::new(1).put::<TaskRewards>();

        let _weight =
            <TaskRewards as OnRuntimeUpgrade>::on_runtime_upgrade();

        let task = Tasks::<Test>::get(0u32).expect("task should be migrated");

        assert_eq!(task.creator, 1);
        assert_eq!(task.reward, 10);
        assert_eq!(task.deposit, 10);
        assert_eq!(task.deadline, 100);
        assert_eq!(task.status, TaskStatus::Open);
        assert_eq!(task.max_submissions, 100);
        assert_eq!(task.submission_count, 0);
    });
}
```

运行：

```bash
cargo test -p pallet-task-rewards migration_v1_to_v2_works \
  --features runtime-benchmarks --lib -- --nocapture
```

## 22. 真实 Runtime Upgrade 验证

目标：

```text
V1 链上有旧 TaskInfo
  -> sudo.setCode 上传 V2 wasm
  -> on_runtime_upgrade 执行
  -> 旧数据补 max_submissions / submission_count
```

### 22.1 保存 V2

```bash
git add .
git commit -m "feat(task-rewards): add v2 storage migration"
git tag task-rewards-v2
```

### 22.2 切回 V1 启链

```bash
git checkout task-rewards-v1
SKIP_PALLET_REVIVE_FIXTURES=1 cargo build --release --locked
```

生成 chain spec：

```bash
chain-spec-builder create -t development \
  --relay-chain paseo \
  --para-id 1000 \
  --runtime ./target/release/wbuild/parachain-template-runtime/parachain_template_runtime.compact.compressed.wasm \
  named-preset development
```

启动：

```bash
rm -rf /tmp/task-rewards-upgrade-demo

polkadot-omni-node \
  --chain ./chain_spec.json \
  --dev \
  --base-path /tmp/task-rewards-upgrade-demo
```

### 22.3 V1 链上写旧数据

Polkadot.js Apps：

```text
Alice -> taskRewards.createTask(reward=10, deadline=当前区块+100)
```

查询：

```text
taskRewards.tasks(0)
```

V1 应只显示：

```text
creator
reward
deposit
deadline
status
```

### 22.4 切回 V2 编译新 Wasm

```bash
git checkout task-rewards-v2
```

提高 `runtime/src/lib.rs` 里的：

```rust
spec_version: 3,
```

编译：

```bash
SKIP_PALLET_REVIVE_FIXTURES=1 cargo build --release --locked
```

注意：这里不要重新生成 chain spec。

### 22.5 sudo.setCode 升级

Polkadot.js Apps：

```text
Developer -> Extrinsics
Alice -> sudo.sudo(call)
call = system.setCode(code)
```

上传：

```text
target/release/wbuild/parachain-template-runtime/parachain_template_runtime.compact.compressed.wasm
```

成功事件：

```text
system.CodeUpdated
sudo.Sudid
system.ExtrinsicSuccess
```

如果报：

```text
SpecVersionNeedsToIncrease
```

说明 `spec_version` 没有比链上版本大。

### 22.6 验证迁移结果

刷新 metadata 后查询：

```text
taskRewards.tasks(0)
```

预期：

```text
creator: Alice
reward: 10
deadline: ...
status: Open
maxSubmissions: 100
submissionCount: 0
```

再提交一次：

```text
Bob -> taskRewards.submitTask(0)
```

预期：

```text
submissionCount: 1
```

## 23. 常见问题

### 23.1 `create_task` 参数数量不对

原因：call 从 3 个参数升级成 4 个参数，测试或 benchmark 还没改。

修复：

```rust
create_task(origin, reward, deadline, max_submissions)
```

### 23.2 migration decode old value 失败

常见原因：

- `OldTaskInfo` 字段顺序不对。
- 测试里旧数据类型推断错。
- `unhashed::put` 时写了 `old_task.encode()`。

正确写法：

```rust
let old_task: OldTaskInfo<u64, u64, BlockNumberFor<Test>> = ...;
unhashed::put(&key, &old_task);
```

### 23.3 runtime upgrade 失败

如果报：

```text
SpecVersionNeedsToIncrease
```

修复：

```rust
spec_version += 1
```

重新编译 Wasm 后再 `sudo.setCode`。

## 24. 推荐学习路线

建议顺序：

1. 跑通模板链。
2. 用 Polkadot.js Apps 连接本地链。
3. 写 Counter pallet。
4. Counter 接入 runtime。
5. Polkadot.js 调用 `counter.increment()`。
6. 给 Counter 补单元测试。
7. 写 Task Rewards pallet。
8. 补 Task Rewards 测试。
9. 补 benchmark 和 weights。
10. 做 V1 -> V2 storage migration。
11. 做真实 runtime upgrade 验证。

不要一上来研究 XCM、Cumulus、平行链。先把：

```text
自定义 pallet
  -> runtime 接入
  -> 本地链调用
  -> 单元测试
  -> benchmark
  -> migration
  -> runtime upgrade
```

这一条链路跑通。

## 25. 工程总结

Substrate / FRAME 开发不是只写一个 Rust 模块，而是围绕 pallet 的完整生命周期做工程化交付：

- 用 `Config` 把业务 pallet 和 runtime 里的账户、资产、余额、origin、weight 等能力解耦。
- 用 `Storage` 保存链上状态，并注意存储结构后续升级的兼容性。
- 用 `Call` 暴露 extrinsic，用 `ensure_signed`、`EnsureOrigin` 等方式控制权限。
- 用 `Event` 记录成功路径，方便 Polkadot.js Apps、前端和索引服务观察链上行为。
- 用 `Error` 明确失败原因，避免业务逻辑静默失败。
- 用 mock runtime 和单元测试验证 pallet 的状态变化。
- 用 benchmark 生成 weight，避免 runtime 交易成本失真。
- 用 storage migration 和 runtime upgrade 处理已经上链的数据结构演进。

本仓库的企业任务平台就是在这条链路上继续扩展：从任务状态机开始，逐步接入 `pallet-assets`、`pallet-identity`、`pallet-collective`、`pallet-scheduler` 和 `pallet-nfts`，最终形成一个可以在本地链上手动演示的业务闭环。
