import { CheckCircle2, XCircle } from "lucide-react";
import { useChain } from "../store/chainStore";

export function TxPanel() {
  const { txResult, clearTx } = useChain();
  if (!txResult) return null;

  const failed = txResult.status === "failed";

  return (
    <section className={`tx-panel ${failed ? "failed" : ""}`}>
      <div className="tx-heading">
        <div className="tx-title">
          {failed ? <XCircle size={18} /> : <CheckCircle2 size={18} />}
          <span>交易状态：{txResult.status}</span>
        </div>
        <button className="ghost-button" onClick={clearTx}>
          清除
        </button>
      </div>
      {txResult.hash && <p>Hash：{txResult.hash}</p>}
      {txResult.blockHash && <p>Block：{txResult.blockHash}</p>}
      {txResult.error && <p className="error-text">{txResult.error}</p>}
      {txResult.events.length > 0 && (
        <div className="event-list">
          {txResult.events.map((event, index) => (
            <div className="event-row" key={`${event.section}-${event.method}-${index}`}>
              <strong>
                {event.section}.{event.method}
              </strong>
              <span>{event.data}</span>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}
