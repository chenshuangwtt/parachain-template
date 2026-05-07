# 企业任务平台前端

这是 `parachain-template` 企业任务平台的 React + Vite + TypeScript 前端。它通过 Polkadot.js Extension 签名，并通过 `@polkadot/api` 连接本地链。样式使用 Tailwind CSS。

## 启动

先启动本地链，默认 WebSocket 地址：

```text
ws://127.0.0.1:9944
```

安装依赖并启动前端：

```bash
cd frontend
npm install
npm run dev
```

浏览器打开：

```text
http://localhost:5173
```

## 账户要求

浏览器需要安装 Polkadot.js Extension，并导入开发账户：

- Alice：sudo、assets/nfts owner、committee member
- Bob：任务领取者、提交者、积分和 NFT 获得者
- Charlie / Dave：committee member，用于投票

前端不会内置 `//Alice`、`//Bob` 等 seed。

## 功能

- 连接链和切换 RPC。
- 读取 Extension 账户并选择签名账户。
- 检查 assets、NFT collection、membership、identity 状态。
- 半自动执行初始化 extrinsic。
- 创建任务、领取任务、提交任务。
- 通过 committee propose/vote/close 执行 approve/reject/setCertificateCollectionId。
- 查询积分和 NFT 证书。

## 技术栈

- React
- Vite
- TypeScript
- Tailwind CSS
- @polkadot/api
- @polkadot/extension-dapp

## 注意

`approveTask`、`rejectTask`、`setCertificateCollectionId` 当前需要 committee collective origin。不要用 `sudo.sudo(tasks.setCertificateCollectionId(...))`，否则会 `BadOrigin`。
