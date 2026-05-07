import { RefreshCw } from "lucide-react";
import { useState } from "react";
import { readAssetBalance, readCertificateOwner } from "../lib/chain";
import { useChain } from "../store/chainStore";

export function Certificates() {
  const { api } = useChain();
  const [bobAddress, setBobAddress] = useState("");
  const [collectionId, setCollectionId] = useState("0");
  const [taskId, setTaskId] = useState("0");
  const [assetBalance, setAssetBalance] = useState("-");
  const [certificateOwner, setCertificateOwner] = useState<string | null>(null);
  const [currentCollection, setCurrentCollection] = useState("-");

  const refresh = async () => {
    if (!api) return;
    const [collection, balance, owner] = await Promise.all([
      api.query.tasks.certificateCollectionId(),
      bobAddress ? readAssetBalance(api, bobAddress) : Promise.resolve("-"),
      readCertificateOwner(api, collectionId, taskId),
    ]);
    setCurrentCollection(collection.toString());
    setAssetBalance(balance);
    setCertificateOwner(owner);
  };

  return (
    <div className="page">
      <section className="page-heading">
        <h2>积分和证书</h2>
        <p>approve 成功后检查 Bob 的资产余额，以及 NFT item owner。</p>
      </section>

      <section className="panel">
        <div className="panel-header">
          <h3>查询条件</h3>
          <button onClick={refresh} disabled={!api}>
            <RefreshCw size={16} />
            查询
          </button>
        </div>
        <div className="form-grid">
          <div className="field">
            <label>Bob 地址</label>
            <input value={bobAddress} onChange={(event) => setBobAddress(event.target.value)} />
          </div>
          <div className="field">
            <label>collectionId</label>
            <input value={collectionId} onChange={(event) => setCollectionId(event.target.value)} />
          </div>
          <div className="field">
            <label>taskId / itemId</label>
            <input value={taskId} onChange={(event) => setTaskId(event.target.value)} />
          </div>
        </div>
      </section>

      <div className="metric-grid">
        <div className="metric">
          <span>当前全局 collection</span>
          <strong>{currentCollection}</strong>
        </div>
        <div className="metric">
          <span>assets.account(1, Bob)</span>
          <strong>{assetBalance}</strong>
        </div>
        <div className="metric wide">
          <span>nfts.item(collectionId, taskId).owner</span>
          <strong>{certificateOwner ?? "不存在或证书关闭"}</strong>
        </div>
      </div>
    </div>
  );
}
