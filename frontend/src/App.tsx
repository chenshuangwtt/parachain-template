import { Activity, Award, CheckSquare, ClipboardList, Settings } from "lucide-react";
import { useState } from "react";
import { AccountSelect } from "./components/AccountSelect";
import { TxPanel } from "./components/TxPanel";
import { Certificates } from "./pages/Certificates";
import { Committee } from "./pages/Committee";
import { Dashboard } from "./pages/Dashboard";
import { Setup } from "./pages/Setup";
import { Tasks } from "./pages/Tasks";
import { useChain } from "./store/chainStore";

type PageKey = "dashboard" | "setup" | "tasks" | "committee" | "certificates";

const pages = [
  { key: "dashboard" as const, label: "总览", icon: Activity },
  { key: "setup" as const, label: "初始化", icon: Settings },
  { key: "tasks" as const, label: "任务", icon: ClipboardList },
  { key: "committee" as const, label: "委员会", icon: CheckSquare },
  { key: "certificates" as const, label: "证书", icon: Award },
];

export default function App() {
  const [page, setPage] = useState<PageKey>("dashboard");
  const { endpoint, setEndpoint, connect, connecting, connected, chainName, runtimeVersion, currentBlock, error } =
    useChain();

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <span className="brand-mark">ET</span>
          <div>
            <h1>企业任务平台</h1>
            <p>Substrate runtime demo</p>
          </div>
        </div>
        <nav>
          {pages.map((item) => {
            const Icon = item.icon;
            return (
              <button
                className={page === item.key ? "nav-button active" : "nav-button"}
                key={item.key}
                onClick={() => setPage(item.key)}
              >
                <Icon size={18} />
                <span>{item.label}</span>
              </button>
            );
          })}
        </nav>
      </aside>

      <main className="main">
        <header className="topbar">
          <div className="field rpc-field">
            <label>RPC</label>
            <div className="inline-control">
              <input value={endpoint} onChange={(event) => setEndpoint(event.target.value)} />
              <button onClick={connect} disabled={connecting}>
                {connecting ? "连接中" : "连接"}
              </button>
            </div>
          </div>
          <AccountSelect />
          <div className="chain-meta">
            <span className={connected ? "dot online" : "dot"} />
            <span>{connected ? chainName : "未连接"}</span>
            <span>Runtime {runtimeVersion}</span>
            <span>Block {currentBlock}</span>
          </div>
        </header>

        {error && <div className="alert error">{error}</div>}
        <TxPanel />

        {page === "dashboard" && <Dashboard goTo={setPage} />}
        {page === "setup" && <Setup />}
        {page === "tasks" && <Tasks />}
        {page === "committee" && <Committee />}
        {page === "certificates" && <Certificates />}
      </main>
    </div>
  );
}
