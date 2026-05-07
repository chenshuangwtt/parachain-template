import { ArrowRight, CheckCircle2, Clock, ShieldCheck } from "lucide-react";
import type { Dispatch, SetStateAction } from "react";
import { useEffect, useState } from "react";
import { readSetupState, readTasks } from "../lib/chain";
import { useChain } from "../store/chainStore";

type PageKey = "dashboard" | "setup" | "tasks" | "committee" | "certificates";

export function Dashboard({ goTo }: { goTo: Dispatch<SetStateAction<PageKey>> }) {
  const { api, connected, accounts } = useChain();
  const [taskCount, setTaskCount] = useState(0);
  const [setupReady, setSetupReady] = useState(false);

  useEffect(() => {
    if (!api || !connected) return;
    Promise.all([readTasks(api), readSetupState(api)])
      .then(([tasks, setup]) => {
        setTaskCount(tasks.length);
        setSetupReady(setup.assetExists && setup.collectionExists);
      })
      .catch(() => {
        setTaskCount(0);
        setSetupReady(false);
      });
  }, [api, connected]);

  return (
    <div className="page">
      <section className="page-heading">
        <h2>运行总览</h2>
        <p>先连接本地链和 Polkadot.js Extension，再按初始化、任务、委员会、证书顺序跑完整闭环。</p>
      </section>

      <div className="metric-grid">
        <div className="metric">
          <CheckCircle2 size={22} />
          <span>链连接</span>
          <strong>{connected ? "已连接" : "未连接"}</strong>
        </div>
        <div className="metric">
          <ShieldCheck size={22} />
          <span>初始化</span>
          <strong>{setupReady ? "基础资源存在" : "待检查"}</strong>
        </div>
        <div className="metric">
          <Clock size={22} />
          <span>任务数</span>
          <strong>{taskCount}</strong>
        </div>
        <div className="metric">
          <ShieldCheck size={22} />
          <span>扩展账户</span>
          <strong>{accounts.length}</strong>
        </div>
      </div>

      <section className="workflow">
        <button onClick={() => goTo("setup")}>
          初始化检查 <ArrowRight size={16} />
        </button>
        <button onClick={() => goTo("tasks")}>
          创建和提交任务 <ArrowRight size={16} />
        </button>
        <button onClick={() => goTo("committee")}>
          委员会审核 <ArrowRight size={16} />
        </button>
        <button onClick={() => goTo("certificates")}>
          积分和证书查询 <ArrowRight size={16} />
        </button>
      </section>
    </div>
  );
}
