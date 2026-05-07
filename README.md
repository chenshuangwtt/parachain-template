# Substrate 企业任务平台

这是一个基于 Polkadot SDK / FRAME 的企业任务平台示例项目。项目从官方 parachain template 演进而来，重点展示如何在 runtime 中组合官方 pallet 和自定义 pallet，完成一个可运行的链上业务流程。

当前最终形态支持：

- 企业创建任务，设置奖励、截止区块和是否发 NFT 证书。
- 用户完成 identity 认证后领取任务。
- 用户提交任务后，由 committee 审核。
- 审核通过后自动发放积分资产。
- 任务到期后由 scheduler 自动关闭。
- 审核通过时可按任务配置自动 mint NFT 证书。
- 管理员可通过 committee 动态设置证书 NFT collection。

项目定位是学习和实战演示，不是生产环境代码。生产化还需要补充权限细分、metadata、storage migration、benchmark 完整权重、前端和部署安全策略。

---

## 技术栈

| 模块 | 作用 |
| --- | --- |
| Polkadot SDK / Substrate | 区块链底层框架 |
| FRAME | Runtime pallet 开发框架 |
| `pallet-tasks` | 自定义企业任务业务 pallet |
| `pallet-assets` | 积分资产 |
| `pallet-identity` | 用户身份认证 |
| `pallet-membership` | 委员会成员管理 |
| `pallet-collective` | 委员会提案、投票、执行 |
| `pallet-scheduler` | 截止区块自动关闭任务 |
| `pallet-nfts` | 链上证书 NFT |
| React + Vite + Tailwind CSS | 企业任务平台前端 |
| `@polkadot/api` | 前端连接本地链和读取 runtime metadata |
| Polkadot.js Apps | 本地链交互和手动测试 |

---

## 项目结构

```text
parachain-template/
├── docs/
│   ├── Substrate 企业任务平台.md
│   └── Substrate框架介绍与示例.md
├── frontend/
│   ├── src/
│   ├── package.json
│   └── README.md
├── pallets/
│   ├── counter/
│   ├── task-rewards/
│   ├── tasks/
│   └── template/
├── runtime/
├── node/
├── Cargo.toml
├── chain_spec.json
└── README.md
```

重点文件：

```text
pallets/tasks/src/lib.rs        企业任务 pallet 核心逻辑
pallets/tasks/src/mock.rs       tasks pallet 测试 runtime
pallets/tasks/src/tests.rs      tasks pallet 单元测试
pallets/tasks/src/weights.rs    tasks pallet 权重
runtime/src/configs/mod.rs      runtime pallet 配置
runtime/src/lib.rs              runtime 入口与版本配置
frontend/src/lib/chain.ts       前端链调用封装
frontend/src/pages/Setup.tsx    前端初始化、identity judgement 页面
docs/                           学习和操作文档
```

---

## 阶段分支

项目按阶段保留了分支，方便回看每一步的演进。

| 分支 | 内容 |
| --- | --- |
| `counter-v1-demo` | Counter pallet 入门示例 |
| `task-rewards` | Task Rewards v1 示例 |
| `task-rewards-v2-migration` | Task Rewards storage migration 示例 |
| `phase-1-assets-tasks` | Phase 1：assets + tasks |
| `phase-2-identity` | Phase 2：identity |
| `phase-3-committee-approval` | Phase 3：membership + collective |
| `phase-4-scheduler-deadlines` | Phase 4：scheduler deadline 自动关闭 |
| `phase-5-nft-certificates` | Phase 5：NFT 证书基础和自动 mint |
| `phase-6-dynamic-nft-config` | 当前最终版：动态 certificate collection + 任务级开关 |

常用查看方式：

```bash
git branch
git checkout phase-6-dynamic-nft-config
git log --oneline --decorate -10
```

---

## Tags

部分阶段也保留了 tag，用于固定阶段快照：

```text
counter-v1
counter-v2
task-rewards-v1
task-rewards-v2
tag-phase-1-assets-tasks
tag-phase-2-identity
tag-phase-3-committee-approval
phase-4-scheduler
phase-5-nfts-basic
phase-5-auto-nft-certificates
```

查看 tag：

```bash
git tag -l -n
```

切到某个阶段快照：

```bash
git checkout tag-phase-1-assets-tasks
```

---

## 环境要求

建议在 WSL2 / Ubuntu 中运行。

```text
Ubuntu / WSL2
Rust 1.88.x
Polkadot.js Apps
```

安装基础依赖：

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
rustup toolchain install 1.88.0
rustup default 1.88.0
rustup target add wasm32-unknown-unknown --toolchain 1.88.0
rustup component add rust-src --toolchain 1.88.0
```

---

## 快速开始

克隆项目：

```bash
git clone https://github.com/chenshuangwtt/parachain-template.git
cd parachain-template
```

切到最终阶段分支：

```bash
git checkout phase-6-dynamic-nft-config
```

运行核心测试：

```bash
cargo test -p pallet-tasks
```

检查 runtime：

```bash
cargo check -p parachain-template-runtime
```

完整编译：

```bash
cargo build --release --locked
```

如果 Windows 下 `wasm-opt-sys` 或 wasm 相关依赖编译卡住，优先切到 WSL 中执行。

---

## 本地链运行

安装运行工具：

```bash
cargo install --locked staging-chain-spec-builder
cargo install --locked polkadot-omni-node
```

编译 runtime：

```bash
cargo build --profile production
```

生成 chain spec：

```bash
chain-spec-builder create --relay-chain "rococo-local" --runtime \
  target/release/wbuild/parachain-template-runtime/parachain_template_runtime.wasm \
  named-preset development > chain_spec.json
```

启动 dev chain：

```bash
polkadot-omni-node --chain ./chain_spec.json --dev --dev-block-time 1000
```

连接 Polkadot.js Apps：

```text
https://polkadot.js.org/apps/#/explorer?rpc=ws://127.0.0.1:9944
```

如果你使用的是 parachain 端口或模板生成的其他端口，以本地节点日志中的 WebSocket 地址为准。

---

## 前端应用

仓库包含一个 React + Vite + TypeScript 前端，目录：

```text
frontend/
```

前端功能：

- 默认连接 `ws://127.0.0.1:9944`，页面可切换 RPC。
- 支持 `Dev Accounts` 和 `Polkadot.js Extension` 两种签名模式。
- Dev Accounts 内置本地演示账户：Alice、Bob、Charlie、Dave、Eve、Ferdie。
- 提供初始化页面：创建 assets、创建 NFT collection、设置 committee members、设置 identity、添加 registrar、提供 judgement。
- 提供任务页面：创建任务、领取任务、提交任务、查看任务状态和原始 storage。
- 提供委员会页面：生成 proposal、vote、close，并查询 `reviewCommittee.voting(proposalHash)` 投票进度。
- 提供证书页面：查询 Bob 积分和 NFT certificate owner。

启动前端：

```powershell
cd frontend
npm install
npm run dev
```

访问：

```text
http://localhost:5173
```

默认 Vite 绑定：

```text
0.0.0.0:5173
```

前端本地演示推荐流程：

```text
1. 顶部选择 Dev Accounts -> Alice。
2. Setup 页面点击“自动填入 Dev Alice/Bob/Charlie/Dave 地址”。
3. Alice 执行 sudo.sudo assets.forceCreate(1)。
4. Alice 执行 sudo.sudo nfts.forceCreate(0)。
5. Alice 执行 sudo.sudo reviewMembership.resetMembers。
6. Alice 执行 sudo.sudo reviewMembership.setPrime。
7. 切换 Dev Accounts -> Bob，执行 identity.setIdentity。
8. 切回 Alice，执行 sudo.sudo identity.addRegistrar。
9. 点击“计算 Bob identity hash”。
10. Alice 执行 identity.provideJudgement。
11. Alice 创建任务，Bob claim + submit。
12. Alice/Charlie/Dave 在 Committee 页面 propose/vote/close approveTask。
13. Certificates 页面查询积分和 NFT 证书。
```

注意：Dev Accounts 模式只适合本地 dev chain，不用于生产环境。

---

## Polkadot.js Apps 操作入口

详细页面操作见：

```text
docs/Substrate 企业任务平台.md
```

核心账号约定：

| 账号 | 用途 |
| --- | --- |
| Alice | sudo 签名账户，assets / nfts owner，也可作为 committee member |
| Bob | 任务领取者、提交者、积分和证书获得者 |
| Charlie / Dave | committee member，用于投票 |

核心 origin：

| Origin | 调用方式 | 用途 |
| --- | --- | --- |
| Signed | 普通账户直接签名 | `createTask`、`claimTask`、`submitTask` |
| Root via sudo | `sudo.sudo(...)` | `assets.forceCreate`、`nfts.forceCreate`、`reviewMembership.resetMembers`、`reviewMembership.setPrime` |
| Collective | `committee.propose/vote/close` | `approveTask`、`rejectTask`、`setCertificateCollectionId` |

初始化示例：

```text
sudo.sudo(assets.forceCreate(1, Alice, true, 1))
sudo.sudo(nfts.forceCreate(owner=Alice, config=default))
sudo.sudo(reviewMembership.resetMembers([Alice, Charlie, Dave]))
sudo.sudo(reviewMembership.setPrime(Alice))
```

开启证书任务流程：

```text
Alice Signed:
tasks.createTask(reward=100, deadline=未来区块, certificateEnabled=true)

Bob Signed:
tasks.claimTask(taskId)
tasks.submitTask(taskId)

Committee Collective:
committee.propose/vote/close 执行 tasks.approveTask(taskId)

查询:
assets.account(1, Bob)
nfts.item(collectionId, taskId).owner
```

关闭证书任务流程：

```text
Alice Signed:
tasks.createTask(reward=100, deadline=未来区块, certificateEnabled=false)

Bob Signed:
tasks.claimTask(taskId)
tasks.submitTask(taskId)

Committee Collective:
committee.propose/vote/close 执行 tasks.approveTask(taskId)

查询:
assets.account(1, Bob) 增加
nfts.item(collectionId, taskId) 不存在
```

注意：当前 `AdminOrigin = committee 2/3`，所以不能用：

```text
sudo.sudo(tasks.setCertificateCollectionId(1))
```

这个调用会返回 `BadOrigin`。正确方式是通过 `committee.propose/vote/close` 执行：

```text
tasks.setCertificateCollectionId(1)
```

---

## 文档

建议阅读：

| 文档 | 内容 |
| --- | --- |
| `docs/Substrate框架介绍与示例.md` | Substrate / FRAME 基础、Counter 示例、Task Rewards、benchmark、migration |
| `docs/Substrate 企业任务平台.md` | 企业任务平台阶段实现、关键代码、账号 origin、Polkadot.js Apps 操作流程、常见错误 |

---

## 常见命令

```bash
# 查看当前分支
git branch

# 查看阶段 tag
git tag -l -n

# 运行 tasks pallet 测试
cargo test -p pallet-tasks

# 检查 runtime
cargo check -p parachain-template-runtime

# 格式化
cargo fmt

# 查看最近提交
git log --oneline --decorate -10
```

---

## 常见问题

### `BadOrigin`

先确认调用需要哪种 origin：

- `sudo.sudo(...)` 产生 Root。
- 普通账户签名是 Signed。
- `committee.propose/vote/close` 才会产生 Collective。

当前 `approveTask` 和 `setCertificateCollectionId` 默认需要 Collective。

### `UnknownCollection`

启用 NFT 证书的任务在 approve 时会 mint 到任务创建时快照的 collection。如果 collection 没有提前创建，会失败并回滚积分和任务状态。

### `AlreadyExists`

证书 NFT 的 itemId 使用 taskId。相同 collection 下同一个 taskId 只能 mint 一次。

### `Vec not found`

`no_std` pallet 中需要显式引入：

```rust
use alloc::vec::Vec;
```

### `PalletFeatures::all_enabled()` 不能放 const

`all_enabled()` 不是 const fn，需要用 `Get` 包装或按当前 runtime 配置方式处理。

---

## 当前状态

当前最终分支：

```text
phase-6-dynamic-nft-config
```

当前最终能力：

```text
createTask(reward, deadline, certificateEnabled)
claimTask
submitTask
approveTask
rejectTask
closeTask
setCertificateCollectionId
```

后续可提升方向：

- NFT metadata：任务标题、完成时间、审核人、证书 URI。
- `setCertificateCollectionId` 时预校验 collection 是否存在。
- `AdminOrigin` 支持 Root + committee 组合权限。
- 为已有链数据补 storage version 和 migration。
- 接入 multisig / proxy 做生产权限管理。
