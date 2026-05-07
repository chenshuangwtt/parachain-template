import { useMemo, useState } from "react";
import {
  buildProposal,
  committeeCloseTx,
  committeeProposeTx,
  committeeVoteTx,
  DEFAULT_LENGTH_BOUND,
  proposalHash,
  readVoting,
  type VotingView,
  type ProposalKind,
} from "../lib/chain";
import { useChain } from "../store/chainStore";

export function Committee() {
  const { api, sendTx } = useChain();
  const [kind, setKind] = useState<ProposalKind>("approve");
  const [value, setValue] = useState("");
  const [threshold, setThreshold] = useState("2");
  const [lengthBound, setLengthBound] = useState(String(DEFAULT_LENGTH_BOUND));
  const [proposalIndex, setProposalIndex] = useState("");
  const [manualHash, setManualHash] = useState("");
  const [approveVote, setApproveVote] = useState(true);
  const [voting, setVoting] = useState<VotingView | null>(null);

  const proposal = useMemo(() => {
    if (!api || !value) return null;
    return buildProposal(api, kind, value);
  }, [api, kind, value]);

  const hash = proposal && api ? proposalHash(api, proposal) : "";
  const activeHash = manualHash || hash;

  // voting(hash) reveals the proposal index, ayes/nays, threshold and end block.
  const refreshVoting = async () => {
    if (!api || !activeHash) return;
    const nextVoting = await readVoting(api, activeHash);
    setVoting(nextVoting);
    if (nextVoting.exists && nextVoting.index !== "-") {
      setProposalIndex(nextVoting.index);
    }
  };

  return (
    <div className="page">
      <section className="page-heading">
        <h2>委员会操作</h2>
        <p>approve/reject/setCertificateCollectionId 必须通过 committee propose/vote/close 产生 Collective origin。</p>
      </section>

      <section className="panel">
        <h3>生成 proposal</h3>
        <div className="form-grid">
          <div className="field">
            <label>操作</label>
            <select value={kind} onChange={(event) => setKind(event.target.value as ProposalKind)}>
              <option value="approve">tasks.approveTask(taskId)</option>
              <option value="reject">tasks.rejectTask(taskId)</option>
              <option value="setCollection">tasks.setCertificateCollectionId(collectionId)</option>
            </select>
          </div>
          <div className="field">
            <label>{kind === "setCollection" ? "collectionId" : "taskId"}</label>
            <input value={value} onChange={(event) => setValue(event.target.value)} />
          </div>
          <div className="field">
            <label>threshold</label>
            <input value={threshold} onChange={(event) => setThreshold(event.target.value)} />
          </div>
          <div className="field">
            <label>lengthBound</label>
            <input value={lengthBound} onChange={(event) => setLengthBound(event.target.value)} />
          </div>
        </div>
        <div className="hash-box">
          <span>proposalHash</span>
          <strong>{hash || "-"}</strong>
        </div>
        <button
          disabled={!api || !proposal}
          onClick={() => api && proposal && sendTx(committeeProposeTx(api, proposal, Number(threshold), Number(lengthBound)))}
        >
          committee.propose
        </button>
      </section>

      <section className="panel">
        <h3>投票进度</h3>
        <div className="button-row">
          <button disabled={!api || !activeHash} onClick={refreshVoting}>
            查询 reviewCommittee.voting
          </button>
        </div>
        {voting && (
          <div className="voting-grid">
            <div>
              <span>状态</span>
              <strong>{voting.exists ? "投票中" : "未找到 proposal"}</strong>
            </div>
            <div>
              <span>index</span>
              <strong>{voting.index}</strong>
            </div>
            <div>
              <span>threshold</span>
              <strong>{voting.threshold}</strong>
            </div>
            <div>
              <span>end</span>
              <strong>{voting.end}</strong>
            </div>
            <div>
              <span>ayes</span>
              <strong>{voting.ayes.length}</strong>
            </div>
            <div>
              <span>nays</span>
              <strong>{voting.nays.length}</strong>
            </div>
          </div>
        )}
        {voting?.exists && (
          <pre className="data-box">{JSON.stringify(voting.raw, null, 2)}</pre>
        )}
      </section>

      <section className="panel">
        <h3>vote / close</h3>
        <div className="form-grid">
          <div className="field">
            <label>proposalHash</label>
            <input value={activeHash} onChange={(event) => setManualHash(event.target.value)} placeholder="可从事件复制" />
          </div>
          <div className="field">
            <label>proposalIndex</label>
            <input value={proposalIndex} onChange={(event) => setProposalIndex(event.target.value)} />
          </div>
          <label className="toggle">
            <input type="checkbox" checked={approveVote} onChange={(event) => setApproveVote(event.target.checked)} />
            <span>approve vote</span>
          </label>
        </div>
        <div className="button-row">
          <button
            disabled={!api || !activeHash || !proposalIndex}
            onClick={() => api && sendTx(committeeVoteTx(api, activeHash, proposalIndex, approveVote))}
          >
            committee.vote
          </button>
          <button
            disabled={!api || !activeHash || !proposalIndex}
            onClick={() => api && sendTx(committeeCloseTx(api, activeHash, proposalIndex, Number(lengthBound)))}
          >
            committee.close
          </button>
        </div>
      </section>

      <section className="panel warning-panel">
        <h3>BadOrigin 提醒</h3>
        <p>当前 runtime 中 AdminOrigin 是 committee 2/3。不要用 sudo.sudo(tasks.setCertificateCollectionId(1))。</p>
      </section>
    </div>
  );
}
