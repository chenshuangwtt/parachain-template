import { Keyring } from "@polkadot/keyring";
import type { KeyringPair } from "@polkadot/keyring/types";
import { cryptoWaitReady } from "@polkadot/util-crypto";

export type DevAccount = {
  name: string;
  uri: string;
  address: string;
  pair: KeyringPair;
};

const DEV_URIS = [
  { name: "Alice", uri: "//Alice" },
  { name: "Bob", uri: "//Bob" },
  { name: "Charlie", uri: "//Charlie" },
  { name: "Dave", uri: "//Dave" },
  { name: "Eve", uri: "//Eve" },
  { name: "Ferdie", uri: "//Ferdie" },
];

let cachedAccounts: DevAccount[] | null = null;

// Local dev chains ship with these well-known sr25519 accounts.
// This mode is for demos only; production signing should use an extension or wallet.
export async function loadDevAccounts(): Promise<DevAccount[]> {
  if (cachedAccounts) return cachedAccounts;

  await cryptoWaitReady();

  const keyring = new Keyring({ type: "sr25519" });

  cachedAccounts = DEV_URIS.map((item) => {
    const pair = keyring.addFromUri(item.uri);

    return {
      name: item.name,
      uri: item.uri,
      address: pair.address,
      pair,
    };
  });

  return cachedAccounts;
}
