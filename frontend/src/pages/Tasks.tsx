import { RefreshCw } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import type { TaskView } from "../lib/chain";
import { claimTaskTx, createTaskTx, readTasks, submitTaskTx } from "../lib/chain";
import { useChain } from "../store/chainStore";

export function Tasks() {
  const { api, sendTx } = useChain();
  const [tasks, setTasks] = useState<TaskView[]>([]);
  const [reward, setReward] = useState("100");
  const [deadline, setDeadline] = useState("");
  const [certificateEnabled, setCertificateEnabled] = useState(true);

  const refresh = useCallback(async () => {
    if (!api) return;
    setTasks(await readTasks(api));
  }, [api]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return (
    <div className="page">
      <section className="page-heading">
        <h2>任务流程</h2>
        <p>创建任务使用 Signed origin；领取和提交建议由 Bob 签名。</p>
      </section>

      <section className="panel">
        <h3>创建任务</h3>
        <div className="form-grid">
          <div className="field">
            <label>reward</label>
            <input value={reward} onChange={(event) => setReward(event.target.value)} />
          </div>
          <div className="field">
            <label>deadline block</label>
            <input value={deadline} onChange={(event) => setDeadline(event.target.value)} placeholder="例如当前区块 + 100" />
          </div>
          <label className="toggle">
            <input
              type="checkbox"
              checked={certificateEnabled}
              onChange={(event) => setCertificateEnabled(event.target.checked)}
            />
            <span>certificateEnabled</span>
          </label>
        </div>
        <button
          disabled={!api || !deadline}
          onClick={async () => {
            if (!api) return;
            await sendTx(createTaskTx(api, reward, deadline, certificateEnabled));
            await refresh();
          }}
        >
          tasks.createTask
        </button>
      </section>

      <section className="panel">
        <div className="panel-header">
          <h3>任务列表</h3>
          <button onClick={refresh} disabled={!api}>
            <RefreshCw size={16} />
            刷新
          </button>
        </div>
        <div className="table">
          <div className="table-head task-table">
            <span>ID</span>
            <span>Status</span>
            <span>Reward</span>
            <span>Deadline</span>
            <span>Certificate</span>
            <span>Assignee</span>
            <span>操作</span>
          </div>
          {tasks.map((task) => (
            <div className="table-row task-table" key={task.id}>
              <span>{task.id}</span>
              <span>
                <span className={`status-pill status-${task.status.toLowerCase()}`}>{task.status}</span>
              </span>
              <span>{task.reward}</span>
              <span>{task.deadline}</span>
              <span>{task.certificateCollection ?? "关闭"}</span>
              <span title={task.assignee ?? ""}>{task.assignee ? short(task.assignee) : "-"}</span>
              <span className="row-actions">
                <button
                  disabled={!api}
                  onClick={async () => {
                    if (!api) return;
                    await sendTx(claimTaskTx(api, task.id));
                    await refresh();
                  }}
                >
                  claim
                </button>
                <button
                  disabled={!api}
                  onClick={async () => {
                    if (!api) return;
                    await sendTx(submitTaskTx(api, task.id));
                    await refresh();
                  }}
                >
                  submit
                </button>
              </span>
              <details className="task-details">
                <summary>详情</summary>
                <pre>{JSON.stringify(task.raw, null, 2)}</pre>
              </details>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}

function short(value: string) {
  return `${value.slice(0, 6)}...${value.slice(-6)}`;
}
