# Substrate 企业任务平台

这是一个基于 Polkadot SDK / FRAME 的企业任务平台示例项目。项目目标是把常见 runtime 能力串成一个可运行的业务流程：企业发布任务，认证用户领取并提交，委员会审核通过后发放积分，并可按任务配置自动发 NFT 证书；任务还支持 deadline，到期后由 scheduler 自动关闭。

这份文档面向 GitHub 项目入门和后续维护，不是聊天记录，也不是官方教材。阅读顺序建议是：先跑通项目，再按 Phase 1 到 Phase 5.3 理解每一步为什么改、改了哪些文件、关键代码在哪里、Polkadot.js Apps 页面怎么操作、用哪个账号、需要什么 origin，以及常见错误怎么排查。

---

## 项目功能

- 任务创建：Alice 等企业账号创建任务，设置奖励、截止区块和是否发 NFT 证书。
- 身份认证：Bob 等用户领取任务前必须通过 identity judgement。
- 任务流转：Open -> Claimed -> Submitted -> Approved / Rejected / Closed。
- 积分奖励：审核通过后通过 `pallet-assets` 给任务领取者发积分。
- 委员会审核：`approveTask`、`rejectTask`、`setCertificateCollectionId` 默认走 committee collective origin。
- 到期关闭：`pallet-scheduler` 在 deadline 自动调用 `closeTask`。
- NFT 证书：`pallet-nfts` 支持 approve 成功后按任务开关 mint 证书 NFT。
- 动态证书集合：管理员可以通过 committee 设置当前证书 collection，新任务会快照当时的 collection。

---

## 技术架构

| 模块 | 作用 |
| --- | --- |
| `pallet-tasks` | 自定义业务 pallet，编排任务状态、积分、身份、审核、deadline、NFT 证书。 |
| `pallet-assets` | 积分资产，审核通过后 mint 到 Bob。 |
| `pallet-identity` | 身份认证，未认证用户不能领取任务。 |
| `pallet-membership` | 管理 committee 成员。 |
| `pallet-collective` | 产生 committee collective origin，用于审核和管理证书 collection。 |
| `pallet-scheduler` | 创建任务时注册 deadline，到期自动关闭未完成任务。 |
| `pallet-nfts` | 链上证书 collection 和 item。 |
| `runtime` | 把这些 pallet 接入链，并配置 origin、资产 ID、NFT 默认 collection、scheduler 等参数。 |

核心设计原则：`pallet-tasks` 不重复实现资产、身份、NFT 和委员会能力，而是通过 `Config` 依赖官方 pallet。这样业务 pallet 只负责状态机和流程编排。

---

## 快速开始

以下命令建议在 WSL 中执行。路径按你的环境替换，文档里的代码路径都使用 `parachain-template/` 之后的相对路径。

```bash
cd /root/solana_project/parachain-template
cargo test -p pallet-tasks
cargo check -p parachain-template-runtime
```

如果需要启动本地链，一般流程是先编译节点，再按项目 README 或模板脚本启动本地 dev chain。当前文档重点放在 runtime/pallet 学习和 Polkadot.js Apps 操作，具体启动命令以仓库当前脚本为准。

---

## 演示账号与 Origin

| 账号 / Origin | 用途 |
| --- | --- |
| Alice | 常用 sudo 签名账户；也作为 assets / nfts owner；也可以是 committee member。 |
| Bob | 任务领取者、提交者、积分和证书获得者。 |
| Charlie / Dave | committee member，用于投票达到阈值。 |
| Signed | 普通账户直接签名，例如 `tasks.createTask`、`claimTask`、`submitTask`。 |
| Root via sudo | `sudo.sudo(...)` 产生 Root origin，例如 `assets.forceCreate`、`nfts.forceCreate`、`membership.setMembers`。 |
| Collective | 由 `committee.propose/vote/close` 产生，用于 `approveTask`、`rejectTask`、`setCertificateCollectionId`。 |

注意：当前默认配置里，`AdminOrigin = committee 2/3`。所以 `sudo.sudo(tasks.setCertificateCollectionId(1))` 会 `BadOrigin`，因为 Root 不等于 committee collective origin。要用 sudo 也能执行，需要修改 runtime 的 `AdminOrigin` 配置。

---

## 文档结构

- `0. 总览`：业务流程、模块关系、状态机。
- `1. Substrate / FRAME 必要基础`：理解 pallet、runtime、storage、call、origin。
- `Phase 1 - Phase 5.3`：每阶段的目标、文件路径、关键代码、测试命令、页面操作和常见错误。
- `最终 Polkadot.js Apps 操作总流程`：从初始化资产/NFT/committee/identity，到开启证书、关闭证书、动态 collection。
- `常见错误总表`：`BadOrigin`、`UnknownCollection`、`AlreadyExists`、`Vec not found`、`PalletFeatures::all_enabled` 等。
- `后续提升方向`：metadata、Root + committee 组合权限、storage migration、多 collection 策略、multisig/proxy。

---

## committee 调用命令模板

Polkadot.js Apps 里 committee 调用不是直接点 `tasks.approveTask`，而是三步：`propose -> vote -> close`。

### approveTask 模板

页面位置：Developer -> Extrinsics。

1. Alice 发起 proposal：

```text
签名账户：Alice
Origin：Signed committee member
模块：committee
方法：propose
threshold: 2
proposal: tasks.approveTask(taskId)
lengthBound: proposal 编码长度；不确定时先填一个偏大的值，例如 1024
```

提交后在事件里记录：

```text
committee.Proposed { account, proposal_index, proposal_hash, threshold }
```

后面 vote / close 要用这里的 `proposalHash` 和 `index`。

2. Charlie 或 Dave 投票：

```text
签名账户：Charlie 或 Dave
Origin：Signed committee member
模块：committee
方法：vote
proposal: 上一步 proposal_hash
index: 上一步 proposal_index
approve: true
```

3. 任一 committee member close：

```text
签名账户：Alice / Charlie / Dave
Origin：Signed committee member，close 后产生 Collective origin 执行 proposal
模块：committee
方法：close
proposalHash: 上一步 proposal_hash
index: 上一步 proposal_index
proposalWeightBound: 足够大的 weight，例如当前页面允许的最大值或 MAXIMUM_BLOCK_WEIGHT 附近
lengthBound: 和 propose 时一致或更大，例如 1024
```

预期事件：

```text
committee.Approved
committee.Executed
tasks.TaskApproved
system.ExtrinsicSuccess
```

### setCertificateCollectionId 模板

只把 proposal 换成：

```text
proposal: tasks.setCertificateCollectionId(1)
```

其他步骤仍然是：

```text
committee.propose -> committee.vote -> committee.close
```

注意：不要用 `sudo.sudo(tasks.setCertificateCollectionId(1))`，当前 runtime 会返回 `BadOrigin`。

## 0. 总览

### 0.1 项目最终目标

企业任务平台要完成这条业务链路：

1. 企业/管理员发布任务。
2. 已认证用户领取任务。
3. 用户提交任务。
4. 委员会审核。
5. 审核通过后发积分。
6. 如果任务开启证书，则发 NFT certificate。
7. 如果任务到 deadline 还没完成，则自动关闭。

最终任务状态流：

```text
Open -> Claimed -> Submitted -> Approved
Open / Claimed / Submitted -> Closed
Submitted -> Rejected
```

### 0.2 阶段路线
| 阶段 | 主题 | 新增复杂度 | 完成结果 |
| --- | --- | --- | --- |
| Phase 1 | assets + tasks | 积分资产 + 自定义任务 pallet | approve 后发积分 |
| Phase 2 | identity | 领取前身份认证 | 未认证 Bob 不能 claim |
| Phase 3 | membership + collective | 委员会治理 | approve / reject 走 committee |
| Phase 4 | scheduler | deadline 自动生命周期 | 到期自动 close task |
| Phase 5.1 | pallet-nfts | NFT pallet 接入 | 手动创建 collection 和 mint NFT |
| Phase 5.2 | approve 自动发固定 NFT | 任务完成证书 | approve 后 mint collection 0 / item taskId |
| Phase 5.3 | 动态 certificate collection + 开关 | 动态配置 + 任务级开关 | certificateEnabled 控制是否发 NFT |

### 0.3 账号和 origin 必须分清
| 账号 / Origin | 用途 | 注意 |
| --- | --- | --- |
| Alice | 常用 sudo 签名账户；assets/nfts owner；可作为 committee member | sudo 只能提供 Root，不等于 committee |
| Bob | 任务领取者、提交者、积分和 NFT 获得者 | claim 前必须有 identity judgement |
| Charlie | committee member | 用于 vote |
| Dave | committee member | 用于 vote |
| Signed | 普通账户签名 | createTask/claimTask/submitTask |
| Root via sudo | sudo.sudo(...) | assets.forceCreate、nfts.forceCreate、membership.setMembers |
| Collective | committee.propose/vote/close | approveTask、setCertificateCollectionId |

当前最终权限重点：
```rust
type AdminOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, (), 2, 3>;
type CloseOrigin = frame_system::EnsureRoot<AccountId>;
```

因此：

- `tasks.approveTask`：committee 2/3。
- `tasks.rejectTask`：committee 2/3。
- `tasks.setCertificateCollectionId`：committee 2/3。
- `sudo.sudo(tasks.setCertificateCollectionId(1))` 会 `BadOrigin`。
- `tasks.closeTask` 是 Root，但通常由 scheduler 自动调用。

---

## 1. Substrate / FRAME 必要基础

### 1.1 Node / Runtime / Pallet

```text
Node：链下节点，负责出块、同步、RPC、交易池。
Runtime：链上状态机，决定交易如何执行、状态如何变化。
Pallet：Runtime 里的模块。
FRAME：写 pallet 和 runtime 的框架。
Polkadot SDK：Substrate、Cumulus、XCM 等组件集合。
```

### 1.2 Pallet 文件通常有什么
| 组成 | 作用 | 本项目例子 |
| --- | --- | --- |
| Config | 外部依赖、常量、origin、类型 | Assets、CertificateNfts、AdminOrigin、Scheduler |
| Storage | 链上状态 | Tasks、NextTaskId、CertificateCollectionId |
| Event | 成功事件 | TaskCreated、TaskApproved、CertificateCollectionIdSet |
| Error | 失败原因 | TaskNotFound、IdentityNotVerified、ScheduleTaskFailed |
| Call | 外部 extrinsic | create_task、claim_task、approve_task |
| mock/tests | 单元测试 runtime 和用例 | pallets/tasks/src/mock.rs、tests.rs |
| weights | 交易权重 | pallets/tasks/src/weights.rs |

---

## 2. Phase 1：assets + tasks

### 2.1 阶段目标

1. 接入 pallet-assets 作为积分系统。
2. 新建 pallet-tasks 作为业务 pallet。
3. 跑通 create/claim/submit/approve。
4. approve 成功后给 Bob mint 积分。

### 2.2 本阶段为什么做

这是整个 Pilot 的地基。先不要混入 identity、committee、scheduler、NFT，否则出了错不知道是业务状态机、资产配置还是权限问题。Phase 1 只证明一件事：任务完成后可以发积分。

### 2.3 必要步骤

1. workspace 注册 pallets/tasks。
2. 新建 pallets/tasks/Cargo.toml。
3. 实现 TaskStatus、Task、NextTaskId、Tasks。
4. 实现 create_task、claim_task、submit_task、approve_task。
5. runtime 接入 pallet-assets。
6. runtime 接入 pallet-tasks。
7. 固定 PointAssetId = 1。
8. mock/tests 里初始化资产并测试 approve mint。

### 2.4 涉及文件

| 文件路径 | 用途 |
| --- | --- |
| `Cargo.toml` | workspace 注册 pallet |
| `pallets/tasks/Cargo.toml` | pallet 依赖 |
| `pallets/tasks/src/lib.rs` | 任务业务代码 |
| `pallets/tasks/src/mock.rs` | 测试 runtime |
| `pallets/tasks/src/tests.rs` | 测试用例 |
| `runtime/src/configs/mod.rs` | assets/tasks runtime 配置 |
| `runtime/src/lib.rs` | runtime 注册 pallet |

### 2.5 关键代码

文件路径：`pallets/tasks/src/lib.rs`

作用：任务状态机。

调用账号 / origin：不是 extrinsic。

失败与排查：状态不匹配会导致 TaskNotOpen/TaskNotClaimed/TaskNotSubmitted。

```rust
pub enum TaskStatus {
    Open,
    Claimed,
    Submitted,
    Approved,
    Rejected,
    Closed,
}
```

文件路径：`pallets/tasks/src/lib.rs`

作用：Task 结构，后续阶段继续往里面加 deadline 和 certificate_collection。

调用账号 / origin：不是 extrinsic。

失败与排查：如果链上已有数据，新增字段需要 migration；本地 dev chain 可重置。

```rust
pub struct Task<AccountId, Balance, BlockNumber, CollectionId> {
    pub creator: AccountId,
    pub assignee: Option<AccountId>,
    pub reward: Balance,
    pub deadline: BlockNumber,
    pub certificate_collection: Option<CollectionId>,
    pub status: TaskStatus,
}
```

文件路径：`pallets/tasks/src/lib.rs`

作用：任务存储。

调用账号 / origin：create_task 写入，Chain state 可查询。

失败与排查：taskId 不存在返回 TaskNotFound。

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
    Task<T::AccountId, BalanceOf<T>, BlockNumberFor<T>, CollectionIdOf<T>>,
    OptionQuery,
>;
```

文件路径：`runtime/src/configs/mod.rs`

作用：配置积分资产 ID。

调用账号 / origin：runtime 配置。

失败与排查：assets.forceCreate 没创建资产 1 时 approve 会 Token.UnknownAsset。

```rust
pub type LocalAssetId = u32;
pub type LocalAssetBalance = u128;

parameter_types! {
    pub const PointAssetId: LocalAssetId = 1;
}
```

文件路径：`pallets/tasks/src/lib.rs`

作用：approve 成功后 mint 积分。

调用账号 / origin：当前最终版 approve 需要 Collective。

失败与排查：资产不存在或权限配置错误会失败。

```rust
T::Assets::mint_into(T::PointAssetId::get(), &assignee, reward)?;
```

### 2.6 WSL / cargo 测试

```bash
cd .
cargo test -p pallet-tasks
cargo check -p parachain-template-runtime
```

测试用例应至少覆盖：

1. `create_task_works`
2. `claim_task_works`
3. `submit_task_works`
4. `only_assignee_can_submit`
5. `approve_task_mints_points`
6. `approve_rejects_non_submitted_task`

### 2.7 Polkadot.js Apps 测试流程

#### 创建积分资产 1

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice
- Origin 类型：Root via sudo
- 调用内容：

```text
sudo.sudo(assets.forceCreate(1, Alice, true, 1))
```

- 预期结果：assets.asset(1) 存在。

#### 创建任务

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice
- Origin 类型：Signed
- 调用内容：

```text
tasks.createTask(reward=100, deadline=未来区块, certificateEnabled=false)
```

- 预期结果：TaskCreated，任务状态 Open。

#### Bob 领取任务

- 页面位置：Developer -> Extrinsics
- 签名账户：Bob
- Origin 类型：Signed
- 调用内容：

```text
tasks.claimTask(taskId)
```

- 预期结果：TaskClaimed，assignee=Bob。

#### Bob 提交任务

- 页面位置：Developer -> Extrinsics
- 签名账户：Bob
- Origin 类型：Signed
- 调用内容：

```text
tasks.submitTask(taskId)
```

- 预期结果：TaskSubmitted，状态 Submitted。

#### committee 审核通过

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice / Charlie / Dave
- Origin 类型：Collective
- 调用内容：

```text
committee.propose(
  threshold=2,
  proposal=tasks.approveTask(taskId),
  lengthBound=1024
)

# 从 committee.Proposed 事件里复制 proposal_hash 和 proposal_index
committee.vote(
  proposalHash=proposal_hash,
  index=proposal_index,
  approve=true
)

committee.close(
  proposalHash=proposal_hash,
  index=proposal_index,
  proposalWeightBound=足够大的 weight,
  lengthBound=1024
)
```

- 预期结果：TaskApproved，Bob 积分增加。

#### 查询积分

- 页面位置：Developer -> Chain state
- 签名账户：无需签名
- Origin 类型：Storage read
- 调用内容：

```text
assets.account(1, Bob)
```

- 预期结果：余额增加 100。

### 2.8 常见错误

| 错误 | 原因 | 处理 |
| --- | --- | --- |
| Token.UnknownAsset | 资产 1 没创建 | 先 sudo.sudo(assets.forceCreate(1, Alice, true, 1)) |
| TaskNotSubmitted | 没 submit 就 approve | Bob 先 claim + submit |
| BadOrigin | 当前最终版 approve 需要 committee | 用 committee.propose/vote/close |

### 2.9 完成标准

1. 单元测试通过。
2. Bob 完整 claim/submit/approve 后资产 1 增加。
3. Polkadot.js Apps 能查到任务和资产余额。

---

## 3. Phase 2：identity

### 3.1 阶段目标

1. Bob 领取任务前必须通过身份认证。
2. 未认证用户调用 claimTask 应失败。
3. 认证逻辑通过 trait 注入，不把 identity storage 写死在 tasks 里。

### 3.2 本阶段为什么做

企业任务不能让任意匿名账户领取。Phase 2 只加领取门槛，不改变审核权限，也不引入 scheduler/NFT。

### 3.3 必要步骤

1. runtime 接入 pallet-identity。
2. pallet-tasks 增加 IdentityVerifier trait。
3. Config 增加 type IdentityVerifier。
4. claim_task 里检查 T::IdentityVerifier::is_verified。
5. runtime 实现 IdentityJudgementVerifier。
6. mock/tests 增加未认证不能 claim。

### 3.4 涉及文件

| 文件路径 | 用途 |
| --- | --- |
| `pallets/tasks/src/lib.rs` | IdentityVerifier 和 claim 校验 |
| `runtime/src/configs/mod.rs` | identity 配置和 judgement verifier |
| `pallets/tasks/src/mock.rs` | 测试 verifier |
| `pallets/tasks/src/tests.rs` | 身份测试 |

### 3.5 关键代码

文件路径：`pallets/tasks/src/lib.rs`

作用：身份校验抽象。

调用账号 / origin：runtime 配置注入。

失败与排查：verifier 返回 false 时 claimTask 失败。

```rust
pub trait IdentityVerifier<AccountId> {
    fn is_verified(who: &AccountId) -> bool;
}

#[pallet::config]
pub trait Config: frame_system::Config {
    type IdentityVerifier: crate::IdentityVerifier<Self::AccountId>;
}
```

文件路径：`pallets/tasks/src/lib.rs`

作用：claim 前检查 Bob 是否认证。

调用账号 / origin：Bob Signed。

失败与排查：没有 judgement 返回 IdentityNotVerified。

```rust
pub fn claim_task(origin: OriginFor<T>, task_id: TaskId) -> DispatchResult {
    let who = ensure_signed(origin)?;

    ensure!(
        T::IdentityVerifier::is_verified(&who),
        Error::<T>::IdentityNotVerified
    );

    Tasks::<T>::try_mutate(task_id, |maybe_task| -> DispatchResult {
        let task = maybe_task.as_mut().ok_or(Error::<T>::TaskNotFound)?;
        ensure!(task.status == TaskStatus::Open, Error::<T>::TaskNotOpen);
        ensure!(task.assignee.is_none(), Error::<T>::AlreadyClaimed);
        task.assignee = Some(who.clone());
        task.status = TaskStatus::Claimed;
        Ok(())
    })?;

    Self::deposit_event(Event::TaskClaimed { task_id, assignee: who });
    Ok(())
}
```

文件路径：`runtime/src/configs/mod.rs`

作用：runtime 中按 judgement 判断是否认证。

调用账号 / origin：Bob 需要 identity + judgement。

失败与排查：只有 info 没 judgement 不算认证。

```rust
pub struct IdentityJudgementVerifier;

impl pallet_tasks::IdentityVerifier<AccountId> for IdentityJudgementVerifier {
    fn is_verified(who: &AccountId) -> bool {
        pallet_identity::IdentityOf::<Runtime>::get(who)
            .map(|registration| {
                registration.judgements.iter().any(|(_, judgement)| {
                    matches!(
                        judgement,
                        pallet_identity::Judgement::Reasonable
                            | pallet_identity::Judgement::KnownGood
                    )
                })
            })
            .unwrap_or(false)
    }
}
```

### 3.6 WSL / cargo 测试

```bash
cd .
cargo test -p pallet-tasks
cargo check -p parachain-template-runtime
```

测试用例应至少覆盖：

1. `claim_fails_when_identity_missing`
2. `claim_works_when_identity_verified`
3. `claim_fails_when_task_not_open`
4. `cannot_claim_twice`

### 3.7 Polkadot.js Apps 测试流程

#### Bob 设置 identity

- 页面位置：Developer -> Extrinsics
- 签名账户：Bob
- Origin 类型：Signed
- 调用内容：

```text
identity.setIdentity(info)
```

- 预期结果：identity.identityOf(Bob) 有 registration。

#### 用 JS 计算 Bob identity info hash

有些 identity judgement 调用需要填 Bob identity info 的 hash。可以在 Polkadot.js Apps 的 Developer -> JavaScript 里执行：

```javascript
const bob = '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty';

const identity = await api.query.identity.identityOf(bob);
console.log(identity.toHuman());

const info = identity.unwrap().info;
const hash = api.registry.hash(info.toU8a());

console.log(hash.toHex());
```

使用方式：

1. Bob 先执行 `identity.setIdentity(info)`。
2. 再到 Developer -> JavaScript 执行上面脚本。
3. 复制最后输出的 `hash.toHex()`。
4. Alice / registrar 在 judgement 相关 extrinsic 里填这个 hash。
5. 再查 `identity.identityOf(Bob)`，确认 judgement 变成 `Reasonable` 或 `KnownGood`。

#### 给 Bob judgement

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice / registrar
- Origin 类型：Root via sudo 或 registrar origin
- 调用内容：

```text
identity.provideJudgement(registrarIndex, Bob, Reasonable)
```

- 预期结果：Bob judgement 是 Reasonable 或 KnownGood。

#### Bob 领取任务

- 页面位置：Developer -> Extrinsics
- 签名账户：Bob
- Origin 类型：Signed
- 调用内容：

```text
tasks.claimTask(taskId)
```

- 预期结果：认证后成功；未认证时 IdentityNotVerified。

### 3.8 常见错误

| 错误 | 原因 | 处理 |
| --- | --- | --- |
| IdentityNotVerified | Bob 没有有效 judgement | 给 Bob 添加 Reasonable/KnownGood judgement |
| 设置 identity 仍失败 | 只有 info 没有 judgement | 查 identity.identityOf(Bob).judgements |
| 测试一直失败 | mock verifier 没设置 Bob verified | mock 中显式配置 Bob 认证 |

### 3.9 完成标准

1. 未认证 Bob 不能 claim。
2. 认证 Bob 可以 claim。
3. claim 后任务状态 Claimed。

---

## 4. Phase 3：membership + collective

### 4.1 阶段目标

1. 审核任务从单点 Root 改为 committee 2/3。
2. approveTask/rejectTask 需要 Collective origin。
3. 为后续 setCertificateCollectionId 复用 AdminOrigin。

### 4.2 本阶段为什么做

企业环境里审核不应该长期依赖 sudo。Phase 3 引入治理权限，为后面证书 collection 管理打基础。

### 4.3 必要步骤

1. runtime 接入 pallet-membership。
2. runtime 接入 pallet-collective。
3. 配置 ReviewCommittee。
4. Root 设置 Alice/Charlie/Dave 为 members。
5. pallet_tasks::Config 中 AdminOrigin 改 committee 2/3。
6. Polkadot.js Apps 用 propose/vote/close 执行 approve。

### 4.4 涉及文件

| 文件路径 | 用途 |
| --- | --- |
| `runtime/src/configs/mod.rs` | membership/collective/AdminOrigin |
| `runtime/src/lib.rs` | 注册 collective/membership |
| `pallets/tasks/src/lib.rs` | approve/reject 使用 AdminOrigin |
| `pallets/tasks/src/tests.rs` | BadOrigin 和 committee 测试 |

### 4.5 关键代码

文件路径：`runtime/src/configs/mod.rs`

作用：collective runtime 配置。

调用账号 / origin：committee member 发起 proposal/vote/close。

失败与排查：threshold/weight/length 不对时 proposal 不执行。

```rust
impl pallet_collective::Config for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type MotionDuration = ReviewMotionDuration;
    type MaxProposals = ReviewMaxProposals;
    type MaxMembers = ReviewMaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type SetMembersOrigin = frame_system::EnsureRoot<AccountId>;
    type MaxProposalWeight = ReviewMaxProposalWeight;
    type DisapproveOrigin = frame_system::EnsureRoot<AccountId>;
    type KillOrigin = frame_system::EnsureRoot<AccountId>;
    type Consideration = ();
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}
```

文件路径：`runtime/src/configs/mod.rs`

作用：membership 配置，Root 管理成员。

调用账号 / origin：Alice 通过 sudo 设置 members。

失败与排查：oldCount 不对或非 Root 会失败。

```rust
impl pallet_membership::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AddOrigin = frame_system::EnsureRoot<AccountId>;
    type RemoveOrigin = frame_system::EnsureRoot<AccountId>;
    type SwapOrigin = frame_system::EnsureRoot<AccountId>;
    type ResetOrigin = frame_system::EnsureRoot<AccountId>;
    type PrimeOrigin = frame_system::EnsureRoot<AccountId>;
    type MembershipInitialized = ReviewCommittee;
    type MembershipChanged = ReviewCommittee;
    type MaxMembers = ReviewMaxMembers;
    type WeightInfo = pallet_membership::weights::SubstrateWeight<Runtime>;
}
```

文件路径：`runtime/src/configs/mod.rs`

作用：tasks 管理权限切到 committee 2/3。

调用账号 / origin：approve/reject/setCertificateCollectionId 均需 Collective。

失败与排查：sudo.sudo(...) 不能满足这个 origin。

```rust
type AdminOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, (), 2, 3>;
```

### 4.6 WSL / cargo 测试

```bash
cd .
cargo test -p pallet-tasks
cargo check -p parachain-template-runtime
```

测试用例应至少覆盖：

1. `signed_user_cannot_approve`
2. `root_cannot_approve_when_admin_origin_is_collective`
3. `committee_can_approve`
4. `committee_can_reject`

### 4.7 Polkadot.js Apps 测试流程

#### 设置 committee members

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice
- Origin 类型：Root via sudo
- 调用内容：

```text
sudo.sudo(membership.setMembers([Alice, Charlie, Dave], prime=Alice, oldCount=0))
```

- 预期结果：membership.members() 包含三人。

#### 发起 approve proposal

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice
- Origin 类型：Signed committee member
- 调用内容：

```text
committee.propose(
  threshold=2,
  proposal=tasks.approveTask(taskId),
  lengthBound=1024
)
```

- 预期结果：生成 `committee.Proposed` 事件，记录 `proposal_hash` 和 `proposal_index`，后续 vote/close 都要用。

#### 投票

- 页面位置：Developer -> Extrinsics
- 签名账户：Charlie / Dave
- Origin 类型：Signed committee member
- 调用内容：

```text
committee.vote(
  proposalHash=proposal_hash,
  index=proposal_index,
  approve=true
)
```

- 预期结果：票数达到 2/3。

#### 关闭 proposal

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice / Charlie / Dave
- Origin 类型：Collective execution
- 调用内容：

```text
committee.close(
  proposalHash=proposal_hash,
  index=proposal_index,
  proposalWeightBound=足够大的 weight,
  lengthBound=1024
)
```

- 预期结果：committee.Executed，tasks.TaskApproved。

### 4.8 常见错误

| 错误 | 原因 | 处理 |
| --- | --- | --- |
| BadOrigin | Alice 直接 approve | 改用 committee proposal |
| BadOrigin | sudo.sudo(tasks.approveTask) | Root 不等于 committee |
| proposal 不执行 | 票数不够或 close 参数不够 | 补 vote，增大 weight/length bound |

### 4.9 完成标准

1. committee members 已设置。
2. approve/reject 通过 committee 成功。
3. 普通 Signed 和 sudo 直接包 approve 均失败。

---

## 5. Phase 4：scheduler

### 5.1 阶段目标

任务创建时支持 deadline，到期后自动关闭未完成任务。

### 5.2 本阶段为什么做

Phase 1-3 已经有任务、身份和审核，但任务如果没人处理，会一直停在 Open / Claimed / Submitted。企业任务必须有截止时间，所以引入 `pallet-scheduler` 自动执行生命周期。

### 5.3 必要步骤

1. `runtime/Cargo.toml` 加 `pallet-scheduler`。
2. `runtime/src/lib.rs` 注册 `Scheduler`。
3. `runtime/src/configs/mod.rs` 配置 `pallet_scheduler::Config`。
4. `Task` 增加 `deadline`。
5. `create_task` 增加 `deadline` 参数。
6. `create_task` 里调用 `schedule_named`。
7. 新增 `close_task`，只允许 Root。
8. mock runtime 加 Scheduler。
9. tests 增加到期关闭测试。

### 5.4 涉及文件
| 文件路径 | 用途 |
| --- | --- |
| `runtime/Cargo.toml` | 增加 pallet-scheduler 依赖 |
| `runtime/src/lib.rs` | 注册 Scheduler |
| `runtime/src/configs/mod.rs` | scheduler runtime config |
| `pallets/tasks/src/lib.rs` | deadline、schedule_named、close_task |
| `pallets/tasks/src/mock.rs` | 测试 runtime 注册 Scheduler |
| `pallets/tasks/src/tests.rs` | scheduler 测试 |

### 5.5 关键代码

文件路径：`pallets/tasks/src/lib.rs`

作用：tasks 注入 scheduler 能力。

调用账号 / origin：内部调度，不由用户直接调用。

失败与排查：类型不匹配时会出现 ScheduleNamed / RuntimeCall / OriginCaller 编译错误。

```rust
type CloseOrigin: EnsureOrigin<Self::RuntimeOrigin>;

type ScheduleOrigin: From<frame_system::RawOrigin<Self::AccountId>>;

type Scheduler: ScheduleNamed<
    BlockNumberFor<Self>,
    Self::TaskRuntimeCall,
    Self::ScheduleOrigin,
>;

type TaskRuntimeCall: From<Call<Self>>;
```

文件路径：`pallets/tasks/src/lib.rs`

作用：create_task 增加 deadline 和 certificate_enabled。

调用账号 / origin：Alice Signed 调用。

失败与排查：deadline 小于等于当前块返回 InvalidDeadline。

```rust
pub fn create_task(
    origin: OriginFor<T>,
    reward: BalanceOf<T>,
    deadline: BlockNumberFor<T>,
    certificate_enabled: bool,
) -> DispatchResult {
    let creator = ensure_signed(origin)?;
    let now = frame_system::Pallet::<T>::block_number();
    ensure!(deadline > now, Error::<T>::InvalidDeadline);
    let task_id = NextTaskId::<T>::get();
    // build task and schedule close_task
    Ok(())
}
```

文件路径：`pallets/tasks/src/lib.rs`

作用：创建任务时注册 deadline 自动关闭。

调用账号 / origin：由 create_task 内部执行；调度后的 close_task 使用 Root。

失败与排查：scheduler 返回错误时映射为 ScheduleTaskFailed。

```rust
let close_call: <T as Config>::TaskRuntimeCall =
    Call::<T>::close_task { task_id }.into();

T::Scheduler::schedule_named(
    Self::close_schedule_name(task_id),
    DispatchTime::At(deadline),
    None,
    63,
    T::ScheduleOrigin::from(frame_system::RawOrigin::Root),
    close_call,
)
.map_err(|_| Error::<T>::ScheduleTaskFailed)?;
```

文件路径：`pallets/tasks/src/lib.rs`

作用：到期关闭任务。

调用账号 / origin：Root；正常由 scheduler 调用。

失败与排查：任务已 Approved/Rejected/Closed 时返回 TaskAlreadyFinalized。

```rust
pub fn close_task(origin: OriginFor<T>, task_id: TaskId) -> DispatchResult {
    T::CloseOrigin::ensure_origin(origin)?;

    Tasks::<T>::try_mutate(task_id, |maybe_task| -> DispatchResult {
        let task = maybe_task.as_mut().ok_or(Error::<T>::TaskNotFound)?;
        ensure!(
            !matches!(task.status, TaskStatus::Approved | TaskStatus::Rejected | TaskStatus::Closed),
            Error::<T>::TaskAlreadyFinalized
        );
        task.status = TaskStatus::Closed;
        Ok(())
    })?;

    Self::deposit_event(Event::TaskClosed { task_id });
    Ok(())
}
```

文件路径：`pallets/tasks/src/lib.rs`

作用：scheduler named task 名称。

调用账号 / origin：内部函数，无外部 origin。

失败与排查：no_std 下缺 Vec 会报 Vec not found。

```rust
fn close_schedule_name(task_id: TaskId) -> Vec<u8> {
    (b"tasks-close", task_id).encode()
}
```

文件路径：`runtime/src/configs/mod.rs`

作用：runtime 配置 scheduler。

调用账号 / origin：ScheduleOrigin 是 Root。

失败与排查：MaximumWeight/MaxScheduledPerBlock 太小可能导致 schedule 失败。

```rust
parameter_types! {
    pub const SchedulerMaxScheduledPerBlock: u32 = 50;
}

impl pallet_scheduler::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type PalletsOrigin = OriginCaller;
    type RuntimeCall = RuntimeCall;
    type MaximumWeight = ReviewMaxProposalWeight;
    type ScheduleOrigin = frame_system::EnsureRoot<AccountId>;
    type MaxScheduledPerBlock = SchedulerMaxScheduledPerBlock;
    type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
    type OriginPrivilegeCmp = frame_support::traits::EqualPrivilegeOnly;
    type Preimages = ();
    type BlockNumberProvider = System;
}
```

### 5.6 WSL / cargo 测试
```bash
cargo test -p pallet-tasks scheduler
cargo test -p pallet-tasks
cargo check -p parachain-template-runtime
```

测试要覆盖：

1. Open 到期 Closed。
2. Claimed 到期 Closed。
3. Submitted 到期 Closed。
4. Approved 到期保持 Approved。
5. Rejected 到期保持 Rejected。
6. deadline <= now 返回 InvalidDeadline。

### 5.7 Polkadot.js Apps 测试流程
#### 创建短 deadline 任务

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice
- Origin 类型：Signed
- 调用内容：

```text
tasks.createTask(reward=100, deadline=当前区块+10, certificateEnabled=false)
```

- 预期结果：TaskCreated，任务状态 Open。

#### 查看 scheduler agenda

- 页面位置：Developer -> Chain state
- 签名账户：无需签名
- Origin 类型：Storage read
- 调用内容：

```text
scheduler.agenda(deadline)
```

- 预期结果：能看到计划任务。

#### 等待 deadline 后查询任务

- 页面位置：Developer -> Chain state
- 签名账户：无需签名
- Origin 类型：Storage read
- 调用内容：

```text
tasks.tasks(taskId)
```

- 预期结果：状态变 Closed。

#### Submitted 到期测试

- 页面位置：Developer -> Extrinsics + Chain state
- 签名账户：Alice / Bob
- Origin 类型：Signed
- 调用内容：

```text
tasks.createTask(...)
tasks.claimTask(taskId)
tasks.submitTask(taskId)
等待 deadline 后查 tasks.tasks(taskId)
```

- 预期结果：状态变 Closed。

#### Approved 不被关闭测试

- 页面位置：Developer -> Extrinsics + Chain state
- 签名账户：Alice / Bob / committee
- Origin 类型：Signed + Collective
- 调用内容：

```text
createTask -> claimTask -> submitTask -> committee.propose(
  threshold=2,
  proposal=tasks.approveTask(taskId),
  lengthBound=1024
)

# 从 committee.Proposed 事件里复制 proposal_hash 和 proposal_index
committee.vote(
  proposalHash=proposal_hash,
  index=proposal_index,
  approve=true
)

committee.close(
  proposalHash=proposal_hash,
  index=proposal_index,
  proposalWeightBound=足够大的 weight,
  lengthBound=1024
) -> 等待 deadline
```

- 预期结果：状态保持 Approved。

### 5.8 本阶段踩坑
| 错误 | 原因 | 最终处理 |
| --- | --- | --- |
| Vec not found | no_std 下没有默认 Vec | `extern crate alloc; use alloc::vec::Vec;` |
| ScheduleNamed 类型不匹配 | scheduler 实际需要 RuntimeCall / OriginCaller | 统一 `TaskRuntimeCall = RuntimeCall`、`ScheduleOrigin = OriginCaller` |
| `?` 不能转 DispatchError | schedule_named 返回错误类型不同 | `.map_err(|_| Error::<T>::ScheduleTaskFailed)?` |
| mock 找不到 Scheduler | 测试 runtime 没注册 scheduler | mock.rs 加 Scheduler pallet 和 Config |
| 到期没关闭 | agenda 没注册或 deadline 没到 | 查 `scheduler.agenda(deadline)` |

### 5.9 完成标准

1. Apps 能看到 scheduler。
2. createTask 后 scheduler.agenda(deadline) 有任务。
3. 到期自动 close。
4. 已终态任务不被覆盖。
---

## 6. Phase 5.1：接入 pallet-nfts

### 6.1 阶段目标

只接入 `pallet-nfts`，页面能看到 nfts，能手动创建 collection，能手动 mint NFT 给 Bob。

### 6.2 为什么先只接入 nfts

NFT pallet 配置项多。如果和 tasks 自动 mint 一起做，错误来源太多。先单独验证 NFT pallet 正常，再把它接到 approve_task。

### 6.3 必要步骤

1. runtime/Cargo.toml 加 `pallet-nfts`。
2. runtime/src/lib.rs 注册 `Nfts`。
3. runtime/src/configs/mod.rs 配置 NFT 类型和参数。
4. Polkadot.js Apps 手动 forceCreate collection。
5. 手动 mint NFT 给 Bob。

### 6.4 涉及文件
| 文件路径 | 用途 |
| --- | --- |
| `runtime/Cargo.toml` | 增加 pallet-nfts 依赖 |
| `runtime/src/lib.rs` | 注册 Nfts |
| `runtime/src/configs/mod.rs` | pallet_nfts::Config |

### 6.5 关键代码

文件路径：`runtime/src/configs/mod.rs`

作用：NFT ID 类型。

调用账号 / origin：runtime 配置。

失败与排查：tasks 中 collection/item 类型要匹配。

```rust
pub type NftCollectionId = u32;
pub type NftItemId = u32;
```

文件路径：`runtime/src/configs/mod.rs`

作用：修复 all_enabled 不能放 const 的问题。

调用账号 / origin：runtime 配置。

失败与排查：直接 `pub const NftFeatures = PalletFeatures::all_enabled()` 会 E0015。

```rust
pub struct NftFeatures;

impl frame_support::traits::Get<pallet_nfts::PalletFeatures> for NftFeatures {
    fn get() -> pallet_nfts::PalletFeatures {
        pallet_nfts::PalletFeatures::all_enabled()
    }
}
```

文件路径：`runtime/src/configs/mod.rs`

作用：NFT pallet runtime 配置。

调用账号 / origin：forceCreate 需要 Root via sudo；普通 create 是 Signed。

失败与排查：collection 不存在时 mint 会 UnknownCollection。

```rust
impl pallet_nfts::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type CollectionId = NftCollectionId;
    type ItemId = NftItemId;
    type Currency = Balances;
    type ForceOrigin = frame_system::EnsureRoot<AccountId>;
    type CreateOrigin = frame_system::EnsureSigned<AccountId>;
    type Locker = ();
    type Features = NftFeatures;
    type OffchainSignature = sp_runtime::MultiSignature;
    type OffchainPublic = sp_runtime::MultiSigner;
    type WeightInfo = pallet_nfts::weights::SubstrateWeight<Runtime>;
    type BlockNumberProvider = System;
}
```

### 6.6 Polkadot.js Apps 测试流程
#### 确认 nfts 模块存在

- 页面位置：Developer -> Extrinsics
- 签名账户：无需签名
- Origin 类型：Metadata
- 调用内容：

```text
下拉模块列表，确认存在 nfts
```

- 预期结果：能看到 nfts。

#### 创建 collection 0

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice
- Origin 类型：Root via sudo
- 调用内容：

```text
sudo.sudo(nfts.forceCreate(owner=Alice, config=default))
```

- 预期结果：nfts.collection(0) 存在。

#### 手动 mint NFT 给 Bob

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice
- Origin 类型：Signed 或 NFT owner 权限
- 调用内容：

```text
nfts.mint(collection=0, item=0, mintTo=Bob, witnessData=...)
```

- 预期结果：nfts.item(0,0).owner = Bob。

### 6.7 常见错误
| 错误 | 原因 | 处理 |
| --- | --- | --- |
| PalletFeatures::all_enabled E0015 | 非 const fn 放进 const | 用 NftFeatures 实现 Get |
| Apps 看不到 nfts | runtime 未注册或 metadata 旧 | 重新 build/启动/刷新 metadata |
| collection id 不是 0 | 之前创建过 collection | 以 chain state 实际 ID 为准 |

---

## 7. Phase 5.2：approve 自动发固定 NFT

### 7.1 阶段目标

approve 成功后自动发 NFT 证书。固定方案：

```text
collectionId = 0
itemId = taskId
owner = assignee / Bob
```

### 7.2 为什么先固定

固定 collection 方案不引入额外 storage 和管理接口，可以先验证自动发证书链路。动态 collection 放到 Phase 5.3。

### 7.3 必要步骤

1. pallet-tasks Config 增加 CertificateNfts。
2. approve_task 用 transaction。
3. do_approve_task 先 mint 积分，再 mint NFT。
4. NFT mint 失败时整体回滚。
5. mock runtime 接入 nfts。
6. tests 增加成功和失败回滚。

### 7.4 涉及文件
| 文件路径 | 用途 |
| --- | --- |
| `pallets/tasks/src/lib.rs` | approve 自动 mint NFT |
| `pallets/tasks/src/mock.rs` | mock nfts |
| `pallets/tasks/src/tests.rs` | NFT 和 rollback 测试 |
| `runtime/src/configs/mod.rs` | tasks 绑定 Nfts |

文件路径：`pallets/tasks/src/lib.rs`

作用：approve 外层事务，避免部分成功。

调用账号 / origin：Collective。

失败与排查：NFT 失败时 rollback。

```rust
pub fn approve_task(origin: OriginFor<T>, task_id: TaskId) -> DispatchResult {
    T::AdminOrigin::ensure_origin(origin)?;

    frame::deps::frame_support::storage::with_transaction(|| {
        let result = Self::do_approve_task(task_id);
        if result.is_ok() {
            frame::deps::sp_runtime::TransactionOutcome::Commit(result)
        } else {
            frame::deps::sp_runtime::TransactionOutcome::Rollback(result)
        }
    })
}
```

文件路径：`pallets/tasks/src/lib.rs`

作用：mint NFT 证书。

调用账号 / origin：committee.propose(
  threshold=2,
  proposal=tasks.approveTask(taskId),
  lengthBound=1024
)

# 从 committee.Proposed 事件里复制 proposal_hash 和 proposal_index
committee.vote(
  proposalHash=proposal_hash,
  index=proposal_index,
  approve=true
)

committee.close(
  proposalHash=proposal_hash,
  index=proposal_index,
  proposalWeightBound=足够大的 weight,
  lengthBound=1024
) 间接触发。

失败与排查：UnknownCollection/AlreadyExists 会导致 approve 回滚。

```rust
T::CertificateNfts::mint_into(
    &collection_id,
    &task_id,
    &assignee,
    &T::CertificateItemConfig::default(),
    true,
)?;
```

### 7.5 Polkadot.js Apps 测试流程
#### 创建 collection 0

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice
- Origin 类型：Root via sudo
- 调用内容：

```text
sudo.sudo(nfts.forceCreate(owner=Alice, config=default))
```

- 预期结果：nfts.collection(0) 存在。

#### 创建启用证书任务

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice
- Origin 类型：Signed
- 调用内容：

```text
tasks.createTask(reward=100, deadline=未来区块, certificateEnabled=true)
```

- 预期结果：任务创建成功。

#### Bob claim + submit

- 页面位置：Developer -> Extrinsics
- 签名账户：Bob
- Origin 类型：Signed
- 调用内容：

```text
tasks.claimTask(taskId)
tasks.submitTask(taskId)
```

- 预期结果：任务状态 Submitted。

#### committee.propose(
  threshold=2,
  proposal=tasks.approveTask(taskId),
  lengthBound=1024
)

# 从 committee.Proposed 事件里复制 proposal_hash 和 proposal_index
committee.vote(
  proposalHash=proposal_hash,
  index=proposal_index,
  approve=true
)

committee.close(
  proposalHash=proposal_hash,
  index=proposal_index,
  proposalWeightBound=足够大的 weight,
  lengthBound=1024
)

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice / Charlie / Dave
- Origin 类型：Collective
- 调用内容：

```text
committee.propose(
  threshold=2,
  proposal=tasks.approveTask(taskId),
  lengthBound=1024
)

# 从 committee.Proposed 事件里复制 proposal_hash 和 proposal_index
committee.vote(
  proposalHash=proposal_hash,
  index=proposal_index,
  approve=true
)

committee.close(
  proposalHash=proposal_hash,
  index=proposal_index,
  proposalWeightBound=足够大的 weight,
  lengthBound=1024
)
```

- 预期结果：Bob 获得积分和 NFT。

#### 查询 NFT

- 页面位置：Developer -> Chain state
- 签名账户：无需签名
- Origin 类型：Storage read
- 调用内容：

```text
nfts.item(0, taskId)
```

- 预期结果：owner 是 Bob。

### 7.6 常见错误
| 错误 | 原因 | 处理 |
| --- | --- | --- |
| UnknownCollection | collection 0 未创建 | 先 sudo.sudo(nfts.forceCreate(...)) |
| AlreadyExists | itemId = taskId 已存在 | 换新任务或重置 dev chain |
| 积分发了但 NFT 没有 | approve 没事务 | 必须 with_transaction |

---

## 8. Phase 5.3：动态 certificate collection + 任务级开关

### 8.1 阶段目标

这是当前最终形态：管理员可设置默认证书 collection，创建任务时可选择是否发证书。启用证书时，任务快照当前 collection；关闭证书时，只发积分不发 NFT。

### 8.2 必要步骤

1. 新增 CertificateCollectionId storage。
2. Config 增加 DefaultCertificateCollectionId。
3. Task 增加 certificate_collection: Option<CollectionId>。
4. create_task 增加 certificate_enabled: bool。
5. 新增 set_certificate_collection_id。
6. approve_task 根据任务快照决定是否 mint NFT。
7. tests 覆盖开启、关闭、动态 collection、失败回滚。

### 8.3 涉及文件
| 文件路径 | 用途 |
| --- | --- |
| `pallets/tasks/src/lib.rs` | storage/create/approve/set collection |
| `runtime/src/configs/mod.rs` | 默认 collection 和 tasks config |
| `pallets/tasks/src/mock.rs` | mock nfts/scheduler/identity |
| `pallets/tasks/src/tests.rs` | Phase 5.3 测试 |

文件路径：`pallets/tasks/src/lib.rs`

作用：动态 certificate collection storage。

调用账号 / origin：通过 set_certificate_collection_id 修改。

失败与排查：不预校验 collection 是否存在，approve 时可能 UnknownCollection。

```rust
#[pallet::storage]
#[pallet::getter(fn certificate_collection_id)]
pub type CertificateCollectionId<T: Config> =
    StorageValue<_, CollectionIdOf<T>, ValueQuery, T::DefaultCertificateCollectionId>;
```

文件路径：`pallets/tasks/src/lib.rs`

作用：NFT 证书相关 Config。

调用账号 / origin：runtime 注入 Nfts。

失败与排查：CollectionId/ItemId 类型不一致会编译失败。

```rust
type CertificateCollectionId: Parameter + MaxEncodedLen + Copy;

type CertificateNfts: NftMutate<
    Self::AccountId,
    Self::CertificateItemConfig,
    CollectionId = Self::CertificateCollectionId,
    ItemId = TaskId,
>;

type DefaultCertificateCollectionId: Get<CollectionIdOf<Self>>;
type CertificateItemConfig: Default;
```

文件路径：`pallets/tasks/src/lib.rs`

作用：创建任务时快照 collection。

调用账号 / origin：Alice Signed。

失败与排查：certificate_enabled=false 时保存 None。

```rust
let task = Task {
    creator: creator.clone(),
    assignee: None,
    reward,
    deadline,
    certificate_collection: certificate_enabled.then(CertificateCollectionId::<T>::get),
    status: TaskStatus::Open,
};
```

文件路径：`pallets/tasks/src/lib.rs`

作用：设置默认证书 collection。

调用账号 / origin：Collective；不能 sudo。

失败与排查：sudo.sudo(...) 会 BadOrigin。

```rust
pub fn set_certificate_collection_id(
    origin: OriginFor<T>,
    collection_id: CollectionIdOf<T>,
) -> DispatchResult {
    T::AdminOrigin::ensure_origin(origin)?;
    CertificateCollectionId::<T>::put(collection_id);
    Self::deposit_event(Event::CertificateCollectionIdSet { collection_id });
    Ok(())
}
```

文件路径：`runtime/src/configs/mod.rs`

作用：当前最终 runtime tasks config。

调用账号 / origin：approve/set collection 是 Collective；close 是 Root。

失败与排查：权限不匹配会 BadOrigin。

```rust
parameter_types! {
    pub const PointAssetId: LocalAssetId = 1;
    pub const DefaultCertificateCollectionId: u32 = 0;
}

impl pallet_tasks::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Assets = Assets;
    type PointAssetId = PointAssetId;
    type CertificateCollectionId = NftCollectionId;
    type CertificateNfts = Nfts;
    type DefaultCertificateCollectionId = DefaultCertificateCollectionId;
    type CertificateItemConfig = pallet_nfts::ItemConfig;
    type AdminOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, (), 2, 3>;
    type CloseOrigin = frame_system::EnsureRoot<AccountId>;
    type ScheduleOrigin = OriginCaller;
    type Scheduler = Scheduler;
    type TaskRuntimeCall = RuntimeCall;
    type IdentityVerifier = IdentityJudgementVerifier;
    type WeightInfo = pallet_tasks::weights::SubstrateWeight<Runtime>;
}
```

### 8.4 WSL / cargo 测试
```bash
cargo test -p pallet-tasks
cargo check -p parachain-template-runtime
```

必须覆盖：默认 collection 为 0；committee 可设置 collection；非管理员失败；true 快照 Some；false 保存 None；旧任务不受全局修改影响；开启证书 mint NFT；关闭证书不 mint；UnknownCollection/AlreadyExists 回滚。

### 8.5 Polkadot.js Apps 测试：开启证书
#### Alice 创建开启证书任务

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice
- Origin 类型：Signed
- 调用内容：

```text
tasks.createTask(reward=100, deadline=未来区块, certificateEnabled=true)
```

- 预期结果：certificateCollection = Some(currentCollectionId)。

#### Bob claim + submit

- 页面位置：Developer -> Extrinsics
- 签名账户：Bob
- Origin 类型：Signed
- 调用内容：

```text
tasks.claimTask(taskId)
tasks.submitTask(taskId)
```

- 预期结果：任务状态 Submitted。

#### committee.propose(
  threshold=2,
  proposal=tasks.approveTask(taskId),
  lengthBound=1024
)

# 从 committee.Proposed 事件里复制 proposal_hash 和 proposal_index
committee.vote(
  proposalHash=proposal_hash,
  index=proposal_index,
  approve=true
)

committee.close(
  proposalHash=proposal_hash,
  index=proposal_index,
  proposalWeightBound=足够大的 weight,
  lengthBound=1024
)

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice / Charlie / Dave
- Origin 类型：Collective
- 调用内容：

```text
committee.propose(
  threshold=2,
  proposal=tasks.approveTask(taskId),
  lengthBound=1024
)

# 从 committee.Proposed 事件里复制 proposal_hash 和 proposal_index
committee.vote(
  proposalHash=proposal_hash,
  index=proposal_index,
  approve=true
)

committee.close(
  proposalHash=proposal_hash,
  index=proposal_index,
  proposalWeightBound=足够大的 weight,
  lengthBound=1024
)
```

- 预期结果：Bob 获得积分和 NFT。

#### 查询结果

- 页面位置：Developer -> Chain state
- 签名账户：无需签名
- Origin 类型：Storage read
- 调用内容：

```text
assets.account(1, Bob)
nfts.item(collectionId, taskId)
```

- 预期结果：积分增加；NFT owner 是 Bob。

### 8.6 Polkadot.js Apps 测试：关闭证书
#### Alice 创建关闭证书任务

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice
- Origin 类型：Signed
- 调用内容：

```text
tasks.createTask(reward=100, deadline=未来区块, certificateEnabled=false)
```

- 预期结果：certificateCollection = None。

#### Bob claim + submit

- 页面位置：Developer -> Extrinsics
- 签名账户：Bob
- Origin 类型：Signed
- 调用内容：

```text
tasks.claimTask(taskId)
tasks.submitTask(taskId)
```

- 预期结果：Submitted。

#### committee.propose(
  threshold=2,
  proposal=tasks.approveTask(taskId),
  lengthBound=1024
)

# 从 committee.Proposed 事件里复制 proposal_hash 和 proposal_index
committee.vote(
  proposalHash=proposal_hash,
  index=proposal_index,
  approve=true
)

committee.close(
  proposalHash=proposal_hash,
  index=proposal_index,
  proposalWeightBound=足够大的 weight,
  lengthBound=1024
)

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice / Charlie / Dave
- Origin 类型：Collective
- 调用内容：

```text
committee.propose(
  threshold=2,
  proposal=tasks.approveTask(taskId),
  lengthBound=1024
)

# 从 committee.Proposed 事件里复制 proposal_hash 和 proposal_index
committee.vote(
  proposalHash=proposal_hash,
  index=proposal_index,
  approve=true
)

committee.close(
  proposalHash=proposal_hash,
  index=proposal_index,
  proposalWeightBound=足够大的 weight,
  lengthBound=1024
)
```

- 预期结果：Bob 只获得积分。

#### 查询没有 NFT

- 页面位置：Developer -> Chain state
- 签名账户：无需签名
- Origin 类型：Storage read
- 调用内容：

```text
nfts.item(collectionId, taskId)
```

- 预期结果：item 不存在。

### 8.7 Polkadot.js Apps 测试：动态 collection
#### Alice 创建 collection 1

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice
- Origin 类型：Root via sudo
- 调用内容：

```text
sudo.sudo(nfts.forceCreate(owner=Alice, config=default))
```

- 预期结果：通常得到 collection 1，以 chain state 为准。

#### committee 设置 collection 为 1

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice / Charlie / Dave
- Origin 类型：Collective
- 调用内容：

```text
committee.propose(
  threshold=2,
  proposal=tasks.setCertificateCollectionId(1),
  lengthBound=1024
)

# 从 committee.Proposed 事件里复制 proposal_hash 和 proposal_index
committee.vote(
  proposalHash=proposal_hash,
  index=proposal_index,
  approve=true
)

committee.close(
  proposalHash=proposal_hash,
  index=proposal_index,
  proposalWeightBound=足够大的 weight,
  lengthBound=1024
)
```

- 预期结果：tasks.certificateCollectionId() 返回 1。

#### 新建启用证书任务并 approve

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice / Bob / committee
- Origin 类型：Signed + Collective
- 调用内容：

```text
tasks.createTask(reward=100, deadline=未来区块, certificateEnabled=true)
tasks.claimTask(taskId)
tasks.submitTask(taskId)
committee.propose(
  threshold=2,
  proposal=tasks.approveTask(taskId),
  lengthBound=1024
)

# 从 committee.Proposed 事件里复制 proposal_hash 和 proposal_index
committee.vote(
  proposalHash=proposal_hash,
  index=proposal_index,
  approve=true
)

committee.close(
  proposalHash=proposal_hash,
  index=proposal_index,
  proposalWeightBound=足够大的 weight,
  lengthBound=1024
)
```

- 预期结果：NFT mint 到 collection 1。

#### 查询 collection 1 NFT

- 页面位置：Developer -> Chain state
- 签名账户：无需签名
- Origin 类型：Storage read
- 调用内容：

```text
nfts.item(1, taskId)
```

- 预期结果：owner 是 Bob。

### 8.8 为什么 sudo 设置 collection 会 BadOrigin

错误操作：
```text
sudo.sudo(tasks.setCertificateCollectionId(1))
```

现象：
```text
sudo.Sudid error: BadOrigin
```

原因：
```rust
type AdminOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, (), 2, 3>;
```

Root 不等于 committee collective origin，所以必须走 committee.propose/vote/close。

### 8.9 常见错误
| 错误 | 原因 | 处理 |
| --- | --- | --- |
| BadOrigin | setCertificateCollectionId 用了 sudo | 改用 committee proposal |
| UnknownCollection | 任务快照的 collection 未创建 | 先创建 collection，再创建新任务 |
| AlreadyExists | itemId=taskId 已存在 | 换新 taskId 或重置链 |
| 旧任务 collection 没变 | 任务创建时快照 | 这是正确行为 |
| 关闭证书查不到 NFT | certificateEnabled=false | 只查积分 |

---

## 9. 最终 Polkadot.js Apps 操作总流程

这一节按最终 runtime 写，适合完整演示。

### 9.1 初始化
#### 创建积分资产 1

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice
- Origin 类型：Root via sudo
- 调用内容：

```text
sudo.sudo(assets.forceCreate(1, Alice, true, 1))
```

- 预期结果：assets.asset(1) 存在。

#### 创建 NFT collection 0

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice
- Origin 类型：Root via sudo
- 调用内容：

```text
sudo.sudo(nfts.forceCreate(owner=Alice, config=default))
```

- 预期结果：nfts.collection(0) 存在。

#### 设置 committee members

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice
- Origin 类型：Root via sudo
- 调用内容：

```text
sudo.sudo(membership.setMembers([Alice, Charlie, Dave], prime=Alice, oldCount=0))
```

- 预期结果：membership.members() 包含 Alice/Charlie/Dave。

#### Bob 设置 identity

- 页面位置：Developer -> Extrinsics
- 签名账户：Bob
- Origin 类型：Signed
- 调用内容：

```text
identity.setIdentity(info)
```

- 预期结果：Bob 有 identity registration。

#### 给 Bob judgement

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice / registrar
- Origin 类型：Root via sudo 或 registrar origin
- 调用内容：

```text
identity.provideJudgement(registrarIndex, Bob, Reasonable)
```

- 预期结果：Bob 通过认证。

### 9.2 开启证书完整流程
#### Alice 创建任务

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice
- Origin 类型：Signed
- 调用内容：

```text
tasks.createTask(reward=100, deadline=未来区块, certificateEnabled=true)
```

- 预期结果：任务保存 Some(collectionId)。

#### Bob 完成任务

- 页面位置：Developer -> Extrinsics
- 签名账户：Bob
- Origin 类型：Signed
- 调用内容：

```text
tasks.claimTask(taskId)
tasks.submitTask(taskId)
```

- 预期结果：Submitted。

#### committee.propose(
  threshold=2,
  proposal=tasks.approveTask(taskId),
  lengthBound=1024
)

# 从 committee.Proposed 事件里复制 proposal_hash 和 proposal_index
committee.vote(
  proposalHash=proposal_hash,
  index=proposal_index,
  approve=true
)

committee.close(
  proposalHash=proposal_hash,
  index=proposal_index,
  proposalWeightBound=足够大的 weight,
  lengthBound=1024
)

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice / Charlie / Dave
- Origin 类型：Collective
- 调用内容：

```text
committee.propose(
  threshold=2,
  proposal=tasks.approveTask(taskId),
  lengthBound=1024
)

# 从 committee.Proposed 事件里复制 proposal_hash 和 proposal_index
committee.vote(
  proposalHash=proposal_hash,
  index=proposal_index,
  approve=true
)

committee.close(
  proposalHash=proposal_hash,
  index=proposal_index,
  proposalWeightBound=足够大的 weight,
  lengthBound=1024
)
```

- 预期结果：Approved。

#### 查询结果

- 页面位置：Developer -> Chain state
- 签名账户：无需签名
- Origin 类型：Storage read
- 调用内容：

```text
assets.account(1, Bob)
nfts.item(collectionId, taskId)
```

- 预期结果：积分增加，NFT owner Bob。

### 9.3 关闭证书完整流程
#### Alice 创建任务

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice
- Origin 类型：Signed
- 调用内容：

```text
tasks.createTask(reward=100, deadline=未来区块, certificateEnabled=false)
```

- 预期结果：任务保存 None。

#### Bob 完成任务

- 页面位置：Developer -> Extrinsics
- 签名账户：Bob
- Origin 类型：Signed
- 调用内容：

```text
tasks.claimTask(taskId)
tasks.submitTask(taskId)
```

- 预期结果：Submitted。

#### committee.propose(
  threshold=2,
  proposal=tasks.approveTask(taskId),
  lengthBound=1024
)

# 从 committee.Proposed 事件里复制 proposal_hash 和 proposal_index
committee.vote(
  proposalHash=proposal_hash,
  index=proposal_index,
  approve=true
)

committee.close(
  proposalHash=proposal_hash,
  index=proposal_index,
  proposalWeightBound=足够大的 weight,
  lengthBound=1024
)

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice / Charlie / Dave
- Origin 类型：Collective
- 调用内容：

```text
committee.propose(
  threshold=2,
  proposal=tasks.approveTask(taskId),
  lengthBound=1024
)

# 从 committee.Proposed 事件里复制 proposal_hash 和 proposal_index
committee.vote(
  proposalHash=proposal_hash,
  index=proposal_index,
  approve=true
)

committee.close(
  proposalHash=proposal_hash,
  index=proposal_index,
  proposalWeightBound=足够大的 weight,
  lengthBound=1024
)
```

- 预期结果：Approved，Bob 只获积分。

#### 查询结果

- 页面位置：Developer -> Chain state
- 签名账户：无需签名
- Origin 类型：Storage read
- 调用内容：

```text
assets.account(1, Bob)
nfts.item(collectionId, taskId)
```

- 预期结果：积分增加，NFT 不存在。

### 9.4 动态 collection 完整流程
#### 创建 collection 1

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice
- Origin 类型：Root via sudo
- 调用内容：

```text
sudo.sudo(nfts.forceCreate(owner=Alice, config=default))
```

- 预期结果：nfts.collection(1) 存在。

#### 设置默认 collection

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice / Charlie / Dave
- Origin 类型：Collective
- 调用内容：

```text
committee.propose(
  threshold=2,
  proposal=tasks.setCertificateCollectionId(1),
  lengthBound=1024
)

# 从 committee.Proposed 事件里复制 proposal_hash 和 proposal_index
committee.vote(
  proposalHash=proposal_hash,
  index=proposal_index,
  approve=true
)

committee.close(
  proposalHash=proposal_hash,
  index=proposal_index,
  proposalWeightBound=足够大的 weight,
  lengthBound=1024
)
```

- 预期结果：tasks.certificateCollectionId() = 1。

#### 创建并审核新任务

- 页面位置：Developer -> Extrinsics
- 签名账户：Alice / Bob / committee
- Origin 类型：Signed + Collective
- 调用内容：

```text
tasks.createTask(..., certificateEnabled=true)
Bob claim + submit
committee.propose(
  threshold=2,
  proposal=tasks.approveTask(taskId),
  lengthBound=1024
)

# 从 committee.Proposed 事件里复制 proposal_hash 和 proposal_index
committee.vote(
  proposalHash=proposal_hash,
  index=proposal_index,
  approve=true
)

committee.close(
  proposalHash=proposal_hash,
  index=proposal_index,
  proposalWeightBound=足够大的 weight,
  lengthBound=1024
)
```

- 预期结果：NFT 发到 collection 1。

#### 查询 NFT

- 页面位置：Developer -> Chain state
- 签名账户：无需签名
- Origin 类型：Storage read
- 调用内容：

```text
nfts.item(1, taskId)
```

- 预期结果：owner Bob。

---

## 10. 常见错误总表
| 分类 | 错误 | 原因 | 修复 |
| --- | --- | --- | --- |
| Origin | BadOrigin | Signed/Root/Collective 混用 | 按 call 要求选择 sudo 或 committee |
| Origin | sudo setCertificateCollectionId BadOrigin | AdminOrigin 是 committee 2/3 | committee.propose/vote/close |
| Assets | Token.UnknownAsset | 资产 1 未创建 | sudo.sudo(assets.forceCreate(1,Alice,true,1)) |
| Identity | IdentityNotVerified | Bob 无 judgement | 给 Bob Reasonable/KnownGood |
| Scheduler | Vec not found | no_std 未导入 Vec | extern crate alloc; use alloc::vec::Vec; |
| Scheduler | ScheduleTaskFailed | scheduler 配置/name/limit 问题 | 查 scheduler.agenda(deadline) |
| NFT | PalletFeatures::all_enabled E0015 | 非 const fn 放 const | 用 Get 包装 |
| NFT | UnknownCollection | collection 未创建 | 先 nfts.forceCreate |
| NFT | AlreadyExists | itemId=taskId 已存在 | 换任务或重置 dev chain |
| Runtime | Windows wasm-opt-sys | Windows 工具链问题 | 用 WSL cargo check |

---

## 11. WSL 命令清单
```bash
cd .
cargo fmt
cargo test -p pallet-tasks
cargo check -p parachain-template-runtime
```

如果只是改 pallet-tasks，优先：
```bash
cargo test -p pallet-tasks
```

---

## 12. 阶段历史记录摘要

### 12.1 Phase 1 历史

最开始目标不是做完整平台，而是把 assets + tasks 跑通。这个阶段的关键是创建资产 1，并证明 approve 后能给 Bob mint 积分。早期可以用 Root 审核，但最终 runtime 已经演进到 committee 审核，所以文档中按最终权限写操作。

### 12.2 Phase 2 历史

identity 阶段解决的是“谁能领取任务”。关键踩坑是：Bob 调了 setIdentity 不等于通过认证，必须有 judgement。最终 verifier 只认 Reasonable 或 KnownGood。

### 12.3 Phase 3 历史

committee 阶段解决“谁能审核任务”。从这一阶段开始，AdminOrigin 变成 committee 2/3。后续 setCertificateCollectionId 的 BadOrigin 也是这个设计导致的正常结果。

### 12.4 Phase 4 历史

scheduler 是今晚重点之一。主要踩坑包括 ScheduleNamed 类型、mock runtime 注册 Scheduler、Vec not found、schedule_named 错误不能直接用 ?。最终方案是 createTask 注册 closeTask，deadline 到期用 Root origin 自动关闭。

### 12.5 Phase 5 历史

Phase 5 先选择 nfts，而不是 multisig/proxy。顺序是：先只接入 pallet-nfts，再 approve 自动发固定 NFT，最后做动态 CertificateCollectionId 和 certificateEnabled 开关。最终 approve 逻辑必须事务性，避免积分和 NFT 部分成功。

---

## 13. 后续提升方向

1. Root + committee 双权限：本地调试可更方便，但生产要谨慎。
2. NFT metadata：任务标题、完成时间、审核人、证书 URI。
3. setCertificateCollectionId 时预校验 collection 是否存在。
4. storage migration：如果不重置 dev chain，需要给 Task 新字段迁移。
5. 多 collection 策略：按企业、任务类型或等级选择 collection。
6. multisig/proxy：练公司账户多签和代理权限。

---

## 14. 验收清单

### 14.1 代码层

1. cargo test -p pallet-tasks 通过。
2. WSL 中 cargo check -p parachain-template-runtime 通过。
3. createTask 有 deadline 和 certificateEnabled。
4. approveTask 事务性发积分和可选 NFT。
5. closeTask 由 scheduler 到期调用。
6. setCertificateCollectionId 需要 committee。

### 14.2 操作层

1. Alice sudo 创建 assets 1。
2. Alice sudo 创建 NFT collection。
3. Alice sudo 设置 committee members。
4. Bob 有 identity judgement。
5. Alice createTask。
6. Bob claim + submit。
7. committee.propose(
  threshold=2,
  proposal=tasks.approveTask(taskId),
  lengthBound=1024
)

# 从 committee.Proposed 事件里复制 proposal_hash 和 proposal_index
committee.vote(
  proposalHash=proposal_hash,
  index=proposal_index,
  approve=true
)

committee.close(
  proposalHash=proposal_hash,
  index=proposal_index,
  proposalWeightBound=足够大的 weight,
  lengthBound=1024
)。
8. 开启证书时 Bob 有积分和 NFT。
9. 关闭证书时 Bob 只有积分。
10. 动态 collection 后 NFT 发到新 collection。

### 14.3 理解层

1. 能区分 Signed、Root via sudo、Collective。
2. 能解释 sudo setCertificateCollectionId 为什么 BadOrigin。
3. 能解释 scheduler 为什么用 Root 调 closeTask。
4. 能解释 certificate collection 的快照语义。
5. 能解释 NFT mint 失败为什么积分也回滚。

