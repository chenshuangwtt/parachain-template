import { Users } from "lucide-react";
import { useEffect } from "react";
import { useChain } from "../store/chainStore";

export function AccountSelect() {
  const {
    signerMode,
    setSignerMode,
    devAccounts,
    selectedDevUri,
    setSelectedDevUri,
    loadDevAccountsList,
    accounts,
    selectedAddress,
    setSelectedAddress,
    loadAccounts,
  } = useChain();

  useEffect(() => {
    if (signerMode === "dev" && devAccounts.length === 0) {
      loadDevAccountsList();
    }
  }, [signerMode, devAccounts.length, loadDevAccountsList]);

  return (
    <div className="account-box">
      <div className="field compact">
        <label>签名模式</label>
        <select value={signerMode} onChange={(event) => setSignerMode(event.target.value as "dev" | "extension")}>
          <option value="dev">Dev Accounts</option>
          <option value="extension">Polkadot.js Extension</option>
        </select>
      </div>

      {signerMode === "dev" ? (
        <div className="field compact">
          <label>Dev 账户</label>
          <div className="inline-control">
            <select value={selectedDevUri} onChange={(event) => setSelectedDevUri(event.target.value)}>
              {devAccounts.map((account) => (
                <option key={account.uri} value={account.uri}>
                  {account.name} - {account.uri} - {shortAddress(account.address)}
                </option>
              ))}
            </select>
            <button className="icon-button" onClick={loadDevAccountsList} title="加载 Dev Accounts">
              <Users size={18} />
            </button>
          </div>
        </div>
      ) : (
        <div className="field compact">
          <label>Extension 账户</label>
          <div className="inline-control">
            <select value={selectedAddress} onChange={(event) => setSelectedAddress(event.target.value)}>
              <option value="">未选择账户</option>
              {accounts.map((account) => (
                <option key={account.address} value={account.address}>
                  {account.meta.name ?? "Unnamed"} - {shortAddress(account.address)}
                </option>
              ))}
            </select>
            <button className="icon-button" onClick={loadAccounts} title="读取 Polkadot.js Extension 账户">
              <Users size={18} />
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

function shortAddress(address: string) {
  return `${address.slice(0, 6)}...${address.slice(-6)}`;
}
