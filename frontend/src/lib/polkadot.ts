import { ApiPromise, WsProvider } from "@polkadot/api";
import { web3Accounts, web3Enable, web3FromAddress } from "@polkadot/extension-dapp";
import type { InjectedAccountWithMeta } from "@polkadot/extension-inject/types";
import type { SubmittableExtrinsic } from "@polkadot/api/promise/types";
import type { KeyringPair } from "@polkadot/keyring/types";

export type ExtensionAccount = InjectedAccountWithMeta;

export type SignerSource =
  | { type: "extension"; address: string }
  | { type: "dev"; pair: KeyringPair };

export type TxStatus = "ready" | "broadcast" | "inBlock" | "finalized" | "failed";

export type TxEvent = {
  section: string;
  method: string;
  data: string;
};

export type TxResult = {
  status: TxStatus;
  hash?: string;
  blockHash?: string;
  error?: string;
  events: TxEvent[];
};

export async function connectApi(endpoint: string): Promise<ApiPromise> {
  const provider = new WsProvider(endpoint);
  const api = await ApiPromise.create({ provider });
  await api.isReady;
  return api;
}

export async function loadExtensionAccounts(appName = "Enterprise Task Platform") {
  const extensions = await web3Enable(appName);
  if (extensions.length === 0) {
    throw new Error("没有检测到 Polkadot.js Extension，或用户未授权。");
  }
  return web3Accounts();
}

export async function signAndSendTx(
  api: ApiPromise,
  tx: SubmittableExtrinsic,
  signer: SignerSource,
  onUpdate?: (result: TxResult) => void,
): Promise<TxResult> {
  // Extension accounts need an injected signer; dev accounts sign directly with KeyringPair.
  const injector = signer.type === "extension" ? await web3FromAddress(signer.address) : null;

  return new Promise((resolve, reject) => {
    let unsubscribe: (() => void) | undefined;
    const callback = (result: any) => {
      const events = result.events.map(({ event }: any) => ({
        section: event.section,
        method: event.method,
        data: event.data.toHuman() ? JSON.stringify(event.data.toHuman()) : "",
      }));

      const base: TxResult = {
        status: "ready",
        hash: tx.hash.toHex(),
        events,
      };

      // Surface intermediate states so the UI can show pending/in-block/finalized feedback.
      if (result.status.isBroadcast) {
        onUpdate?.({ ...base, status: "broadcast" });
      }

      if (result.status.isInBlock) {
        onUpdate?.({
          ...base,
          status: "inBlock",
          blockHash: result.status.asInBlock.toHex(),
        });
      }

      if (result.dispatchError) {
        const error = decodeDispatchError(api, result.dispatchError);
        const failed = { ...base, status: "failed" as const, error };
        onUpdate?.(failed);
        unsubscribe?.();
        reject(new Error(error));
        return;
      }

      if (result.status.isFinalized) {
        const finalized = {
          ...base,
          status: "finalized" as const,
          blockHash: result.status.asFinalized.toHex(),
        };
        onUpdate?.(finalized);
        unsubscribe?.();
        resolve(finalized);
      }
    };

    // @polkadot/api exposes overloaded signAndSend signatures that TypeScript cannot
    // narrow cleanly after branching, so the call boundary is intentionally cast here.
    const sendPromise =
      signer.type === "extension"
        ? (tx.signAndSend as any)(signer.address, { signer: injector?.signer }, callback)
        : (tx.signAndSend as any)(signer.pair, callback);

    sendPromise
      .then((unsub: unknown) => {
        if (typeof unsub === "function") {
          unsubscribe = unsub as () => void;
        }
      })
      .catch((error: unknown) => {
        reject(error);
      });
  });
}

export function decodeDispatchError(api: ApiPromise, dispatchError: any): string {
  if (dispatchError.isModule) {
    const decoded = api.registry.findMetaError(dispatchError.asModule);
    return `${decoded.section}.${decoded.name}: ${decoded.docs.join(" ")}`;
  }

  return dispatchError.toString();
}

export function toHuman(value: unknown): string {
  if (value && typeof value === "object" && "toHuman" in value) {
    return JSON.stringify((value as { toHuman: () => unknown }).toHuman(), null, 2);
  }
  return JSON.stringify(value, null, 2);
}
