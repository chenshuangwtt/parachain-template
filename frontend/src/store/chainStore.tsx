import { createContext, useCallback, useContext, useEffect, useMemo, useState } from "react";
import type { ApiPromise } from "@polkadot/api";
import type { ExtensionAccount, TxResult } from "../lib/polkadot";
import { connectApi, loadExtensionAccounts, signAndSendTx } from "../lib/polkadot";
import type { SubmittableExtrinsic } from "@polkadot/api/promise/types";
import type { DevAccount } from "../lib/devAccounts";
import { loadDevAccounts } from "../lib/devAccounts";

const DEFAULT_RPC = "ws://127.0.0.1:9944";

type ChainState = {
  endpoint: string;
  api: ApiPromise | null;
  connected: boolean;
  connecting: boolean;
  chainName: string;
  runtimeVersion: string;
  currentBlock: string;
  accounts: ExtensionAccount[];
  selectedAddress: string;
  currentSignerAddress: string;
  currentSignerName: string;
  txResult: TxResult | null;
  error: string;
  setEndpoint: (value: string) => void;
  connect: () => Promise<void>;
  loadAccounts: () => Promise<void>;
  setSelectedAddress: (value: string) => void;
  sendTx: (tx: SubmittableExtrinsic) => Promise<TxResult>;
  clearTx: () => void;
  signerMode: "dev" | "extension";
  devAccounts: DevAccount[];
  selectedDevUri: string;
  setSignerMode: (value: "dev" | "extension") => void;
  loadDevAccountsList: () => Promise<void>;
  setSelectedDevUri: (value: string) => void;
};

const ChainContext = createContext<ChainState | null>(null);

export function ChainProvider({ children }: { children: React.ReactNode }) {
  const [endpoint, setEndpoint] = useState(DEFAULT_RPC);
  const [api, setApi] = useState<ApiPromise | null>(null);
  const [connected, setConnected] = useState(false);
  const [connecting, setConnecting] = useState(false);
  const [chainName, setChainName] = useState("-");
  const [runtimeVersion, setRuntimeVersion] = useState("-");
  const [currentBlock, setCurrentBlock] = useState("-");
  const [accounts, setAccounts] = useState<ExtensionAccount[]>([]);
  const [selectedAddress, setSelectedAddress] = useState("");
  const [txResult, setTxResult] = useState<TxResult | null>(null);
  const [error, setError] = useState("");
  const [signerMode, setSignerMode] = useState<"dev" | "extension">("dev");
  const [devAccounts, setDevAccounts] = useState<DevAccount[]>([]);
  const [selectedDevUri, setSelectedDevUri] = useState("//Alice");
  // Expose one normalized signer identity so pages do not need to know which mode is active.
  const selectedDevAccount = devAccounts.find((account) => account.uri === selectedDevUri);
  const selectedExtensionAccount = accounts.find((account) => account.address === selectedAddress);
  const currentSignerAddress = signerMode === "dev" ? (selectedDevAccount?.address ?? "") : selectedAddress;
  const currentSignerName =
    signerMode === "dev"
      ? (selectedDevAccount?.name ?? selectedDevUri)
      : (selectedExtensionAccount?.meta.name ?? "Extension account");

  const connect = useCallback(async () => {
    setConnecting(true);
    setError("");
    try {
      if (api) {
        await api.disconnect();
      }
      const nextApi = await connectApi(endpoint);
      setApi(nextApi);
      setConnected(true);
      setChainName((await nextApi.rpc.system.chain()).toString());
      setRuntimeVersion(nextApi.runtimeVersion.specVersion.toString());
    } catch (err) {
      setConnected(false);
      setApi(null);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setConnecting(false);
    }
  }, [api, endpoint]);

  const loadAccounts = useCallback(async () => {
    setError("");
    try {
      const nextAccounts = await loadExtensionAccounts();
      setAccounts(nextAccounts);
      if (!selectedAddress && nextAccounts.length > 0) {
        setSelectedAddress(nextAccounts[0].address);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, [selectedAddress]);

  const loadDevAccountsList = useCallback(async () => {
    const list = await loadDevAccounts();
    setDevAccounts(list);
    if (!selectedDevUri && list.length > 0) {
      setSelectedDevUri(list[0].uri);
    }
  }, [selectedDevUri]);


  const sendTx = useCallback(
    async (tx: SubmittableExtrinsic) => {
      if (!api) throw new Error("链未连接。");
      const signer =
        signerMode === "dev"
          ? {
              type: "dev" as const,
              pair: devAccounts.find((account) => account.uri === selectedDevUri)?.pair,
            }
          : {
              type: "extension" as const,
              address: selectedAddress,
            };

      if (signerMode === "dev" && !signer.pair) {
        throw new Error("请先加载并选择 Dev Account。");
      }

      if (signerMode === "extension" && !selectedAddress) {
        throw new Error("请先选择 Extension 账户。");
      }
      setTxResult({ status: "ready", events: [] });
      // All pages route transactions through this function to keep feedback and errors consistent.
      const result = await signAndSendTx(
        api,
        tx,
        signerMode === "dev"
          ? { type: "dev", pair: signer.pair! }
          : { type: "extension", address: selectedAddress },
        setTxResult,
      );
      setTxResult(result);
      return result;
    },
    [api, selectedAddress, signerMode, devAccounts, selectedDevUri],
  );

  useEffect(() => {
    if (!api) return;

    let unsubscribe: (() => void) | undefined;
    api.rpc.chain.subscribeNewHeads((header) => {
      setCurrentBlock(header.number.toString());
    }).then((unsub) => {
      unsubscribe = unsub;
    });

    return () => {
      unsubscribe?.();
    };
  }, [api]);

  const value = useMemo(
    () => ({
      endpoint,
      api,
      connected,
      connecting,
      chainName,
      runtimeVersion,
      currentBlock,
      accounts,
      selectedAddress,
      currentSignerAddress,
      currentSignerName,
      txResult,
      error,
      setEndpoint,
      connect,
      loadAccounts,
      setSelectedAddress,
      sendTx,
      clearTx: () => setTxResult(null),
      signerMode,
      devAccounts,
      selectedDevUri,
      setSignerMode,
      loadDevAccountsList,
      setSelectedDevUri,
    }),
    [
      endpoint,
      api,
      connected,
      connecting,
      chainName,
      runtimeVersion,
      currentBlock,
      accounts,
      selectedAddress,
      currentSignerAddress,
      currentSignerName,
      txResult,
      error,
      connect,
      loadAccounts,
      sendTx,
      signerMode,
      devAccounts,
      selectedDevUri,
      setSignerMode,
      loadDevAccountsList,
      setSelectedDevUri,
    ],
  );

  return <ChainContext.Provider value={value}>{children}</ChainContext.Provider>;
}

export function useChain() {
  const value = useContext(ChainContext);
  if (!value) throw new Error("useChain must be used inside ChainProvider");
  return value;
}
