import { RefreshCw } from "lucide-react";
import { useState } from "react";
import {
  DEFAULT_CERTIFICATE_COLLECTION_ID,
  provideJudgementTx,
  readIdentityInfoHash,
  readSetupState,
  setIdentityTx,
  sudoAddRegistrarTx,
  sudoForceCreateAssetTx,
  sudoForceCreateNftCollectionTx,
  sudoSetCommitteeMembersTx,
  sudoSetPrimeTx,
} from "../lib/chain";
import { useChain } from "../store/chainStore";
import { StatusBadge } from "../components/StatusBadge";

export function Setup() {
  const { api, currentSignerAddress, currentSignerName, devAccounts, sendTx } = useChain();
  const [bobAddress, setBobAddress] = useState("");
  const [memberText, setMemberText] = useState("");
  const [primeAddress, setPrimeAddress] = useState("");
  const [oldCount, setOldCount] = useState("0");
  const [identityName, setIdentityName] = useState("Bob");
  const [registrarAddress, setRegistrarAddress] = useState("");
  const [registrarIndex, setRegistrarIndex] = useState("0");
  const [identityHash, setIdentityHash] = useState("");
  const [judgement, setJudgement] = useState<"Reasonable" | "KnownGood">("Reasonable");
  const [state, setState] = useState<any>(null);
  const [loading, setLoading] = useState(false);

  const refresh = async () => {
    if (!api) return;
    setLoading(true);
    try {
      setState(await readSetupState(api, bobAddress || undefined));
    } finally {
      setLoading(false);
    }
  };

  const memberList = memberText
    .split(/[,\n]/)
    .map((item) => item.trim())
    .filter(Boolean);
  const alice = devAccounts.find((account) => account.name === "Alice");
  const bob = devAccounts.find((account) => account.name === "Bob");
  const charlie = devAccounts.find((account) => account.name === "Charlie");
  const dave = devAccounts.find((account) => account.name === "Dave");

  // Convenience helper for the standard local demo committee.
  const fillDevCommittee = () => {
    const members = [alice, charlie, dave].filter(Boolean).map((account) => account!.address);
    setMemberText(members.join("\n"));
    setPrimeAddress(alice?.address ?? "");
    setRegistrarAddress(alice?.address ?? "");
    setBobAddress(bob?.address ?? "");
  };

  // Avoid sending users to Polkadot.js Apps just to compute the identity info hash.
  const calculateIdentityHash = async () => {
    if (!api || !bobAddress) return;
    setIdentityHash(await readIdentityInfoHash(api, bobAddress));
  };

  return (
    <div className="page">
      <section className="page-heading">
        <h2>半自动初始化</h2>
        <p>这里提供状态检查和常用 sudo/identity 操作。Dev Accounts 模式下可直接选择 Alice/Bob/Charlie/Dave 签名。</p>
      </section>

      <div className="toolbar">
        <button onClick={refresh} disabled={!api || loading}>
          <RefreshCw size={16} />
          刷新状态
        </button>
      </div>

      <div className="split-grid">
        <section className="panel">
          <h3>状态检查</h3>
          <div className="field">
            <label>Bob 地址</label>
            <input value={bobAddress} onChange={(event) => setBobAddress(event.target.value)} placeholder="用于检查 identity" />
          </div>
          <div className="status-list">
            <StatusBadge ok={Boolean(state?.assetExists)} label={`assets.asset(1): ${state?.assetExists ? "存在" : "未创建"}`} />
            <StatusBadge
              ok={Boolean(state?.defaultCollectionExists)}
              label={`nfts.collection(${DEFAULT_CERTIFICATE_COLLECTION_ID}): ${state?.defaultCollectionExists ? "存在" : "未创建"}`}
            />
            <StatusBadge
              ok={Boolean(state?.activeCollectionExists)}
              label={`active collection(${state?.certificateCollectionId ?? "-"}): ${state?.activeCollectionExists ? "存在" : "未创建"}`}
            />
            <StatusBadge ok={Boolean(state?.members?.length)} label={`committee members: ${state?.members?.length ?? 0}`} />
            <StatusBadge ok={Boolean(state?.prime)} label={`committee prime: ${state?.prime ? "已设置" : "未设置"}`} />
            <StatusBadge ok={Boolean(state?.bobIdentityVerified)} label={`Bob identity: ${state?.bobIdentityVerified ? "已认证" : "未认证"}`} />
          </div>
          <pre className="data-box">{state ? JSON.stringify(state, null, 2) : "点击刷新状态"}</pre>
        </section>

        <section className="panel">
          <h3>初始化操作</h3>
          <p className="hint">当前签名账户：{currentSignerName || "未选择"}</p>
          <button type="button" className="ghost-button" onClick={fillDevCommittee}>
            自动填入 Dev Alice/Bob/Charlie/Dave 地址
          </button>
          <button disabled={!api || !currentSignerAddress} onClick={() => api && sendTx(sudoForceCreateAssetTx(api, currentSignerAddress))}>
            sudo.sudo assets.forceCreate(1)
          </button>
          <button disabled={!api || !currentSignerAddress} onClick={() => api && sendTx(sudoForceCreateNftCollectionTx(api, currentSignerAddress))}>
            sudo.sudo nfts.forceCreate(0)
          </button>

          <div className="field">
            <label>Committee members</label>
            <textarea
              value={memberText}
              onChange={(event) => setMemberText(event.target.value)}
              placeholder="Alice, Charlie, Dave 地址，逗号或换行分隔"
            />
          </div>
          <div className="field">
            <label>Prime</label>
            <input value={primeAddress} onChange={(event) => setPrimeAddress(event.target.value)} placeholder="通常填 Alice 地址" />
          </div>
          <div className="field">
            <label>oldCount</label>
            <input value={oldCount} onChange={(event) => setOldCount(event.target.value)} />
          </div>
          <button
            disabled={!api || memberList.length === 0}
            onClick={() => api && sendTx(sudoSetCommitteeMembersTx(api, memberList, primeAddress || null, oldCount))}
          >
            sudo.sudo reviewMembership.resetMembers
          </button>
          <button disabled={!api || !primeAddress} onClick={() => api && sendTx(sudoSetPrimeTx(api, primeAddress))}>
            sudo.sudo reviewMembership.setPrime
          </button>

          <div className="field">
            <label>identity display</label>
            <input value={identityName} onChange={(event) => setIdentityName(event.target.value)} />
          </div>
          <button disabled={!api} onClick={() => api && sendTx(setIdentityTx(api, identityName))}>
            identity.setIdentity
          </button>
        </section>
      </div>

      <section className="panel">
        <h3>identity registrar / judgement</h3>
        <p className="hint">
          Bob 先用 Bob 账户执行 identity.setIdentity；Alice 用 sudo 添加 registrar；然后 registrar 账户执行 provideJudgement。
        </p>
        <div className="form-grid">
          <div className="field">
            <label>registrar 地址</label>
            <input value={registrarAddress} onChange={(event) => setRegistrarAddress(event.target.value)} placeholder="通常填 Alice 地址" />
          </div>
          <div className="field">
            <label>registrarIndex</label>
            <input value={registrarIndex} onChange={(event) => setRegistrarIndex(event.target.value)} />
          </div>
          <div className="field">
            <label>judgement</label>
            <select value={judgement} onChange={(event) => setJudgement(event.target.value as "Reasonable" | "KnownGood")}>
              <option value="Reasonable">Reasonable</option>
              <option value="KnownGood">KnownGood</option>
            </select>
          </div>
          <div className="field">
            <label>Bob 地址</label>
            <input value={bobAddress} onChange={(event) => setBobAddress(event.target.value)} />
          </div>
          <div className="field">
            <label>identity info hash</label>
            <input value={identityHash} onChange={(event) => setIdentityHash(event.target.value)} placeholder="0x..." />
          </div>
        </div>
        <div className="button-row">
          <button disabled={!api || !bobAddress} onClick={calculateIdentityHash}>
            计算 Bob identity hash
          </button>
          <button disabled={!api || !registrarAddress} onClick={() => api && sendTx(sudoAddRegistrarTx(api, registrarAddress))}>
            sudo.sudo identity.addRegistrar
          </button>
          <button
            disabled={!api || !bobAddress || !identityHash}
            onClick={() => api && sendTx(provideJudgementTx(api, registrarIndex, bobAddress, judgement, identityHash))}
          >
            identity.provideJudgement
          </button>
        </div>
      </section>

      <section className="panel">
        <h3>identity judgement 顺序</h3>
        <div className="split-grid">
          <div className="instruction-list">
            <p>1. 顶部选择 Bob，执行 identity.setIdentity。</p>
            <p>2. 顶部选择 Alice，执行 sudo.sudo identity.addRegistrar。</p>
            <p>3. 点击“计算 Bob identity hash”，自动填入 hash。</p>
            <p>4. 用 registrar 账户执行 identity.provideJudgement。</p>
          </div>
          <div>
            <p className="hint">也可以在 Polkadot.js Apps 的 Developer - JavaScript 手动计算：</p>
            <pre className="code-block">{`const bob = 'Bob 地址';
const identity = await api.query.identity.identityOf(bob);
console.log(identity.toHuman());

const info = identity.unwrap().info;
const hash = api.registry.hash(info.toU8a());

console.log(hash.toHex());`}</pre>
          </div>
        </div>
      </section>
    </div>
  );
}
