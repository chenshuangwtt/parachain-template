import type { ApiPromise } from "@polkadot/api";
import type { SubmittableExtrinsic } from "@polkadot/api/promise/types";

export const POINT_ASSET_ID = 1;
export const DEFAULT_CERTIFICATE_COLLECTION_ID = 0;
export const DEFAULT_COMMITTEE_THRESHOLD = 2;
export const DEFAULT_LENGTH_BOUND = 1024;
export const DEFAULT_WEIGHT_BOUND = {
  refTime: "1000000000000",
  proofSize: "1000000",
};

export type TaskView = {
  id: number;
  creator: string;
  assignee: string | null;
  reward: string;
  deadline: string;
  certificateCollection: string | null;
  status: string;
  raw: Record<string, any>;
};

export type VotingView = {
  exists: boolean;
  ayes: string[];
  nays: string[];
  threshold: string;
  end: string;
  index: string;
  raw: unknown;
};

export type ProposalKind = "approve" | "reject" | "setCollection";

function queryModule(api: ApiPromise, names: string[]) {
  for (const name of names) {
    if ((api.query as any)[name]) return (api.query as any)[name];
  }
  throw new Error(`Runtime query module not found: ${names.join(" / ")}`);
}

function txModule(api: ApiPromise, names: string[]) {
  for (const name of names) {
    if ((api.tx as any)[name]) return (api.tx as any)[name];
  }
  throw new Error(`Runtime tx module not found: ${names.join(" / ")}`);
}

// The runtime uses ReviewMembership/ReviewCommittee, while metadata exposes camelCase
// names in the JS API. Keep fallbacks so older docs or branch variants still work.
function committeeQuery(api: ApiPromise) {
  return queryModule(api, ["reviewCommittee", "committee"]);
}

function membershipTx(api: ApiPromise) {
  return txModule(api, ["reviewMembership", "membership"]);
}

function committeeTx(api: ApiPromise) {
  return txModule(api, ["reviewCommittee", "committee"]);
}

export function createTaskTx(
  api: ApiPromise,
  reward: string | number,
  deadline: string | number,
  certificateEnabled: boolean,
): SubmittableExtrinsic {
  return api.tx.tasks.createTask(reward, deadline, certificateEnabled);
}

export function claimTaskTx(api: ApiPromise, taskId: string | number): SubmittableExtrinsic {
  return api.tx.tasks.claimTask(taskId);
}

export function submitTaskTx(api: ApiPromise, taskId: string | number): SubmittableExtrinsic {
  return api.tx.tasks.submitTask(taskId);
}

export function sudoForceCreateAssetTx(api: ApiPromise, owner: string): SubmittableExtrinsic {
  return api.tx.sudo.sudo(api.tx.assets.forceCreate(POINT_ASSET_ID, owner, true, 1));
}

export function sudoForceCreateNftCollectionTx(api: ApiPromise, owner: string): SubmittableExtrinsic {
  return api.tx.sudo.sudo(api.tx.nfts.forceCreate(owner, defaultNftCollectionConfig(api)));
}

export function sudoSetCommitteeMembersTx(
  api: ApiPromise,
  members: string[],
  prime: string | null,
  oldCount: string | number,
): SubmittableExtrinsic {
  // Current runtime uses pallet-membership resetMembers and not collective setMembers.
  // The collective path is kept only for compatibility with other template variants.
  const collective = committeeTx(api) as any;
  if (typeof collective.setMembers === "function") {
    return api.tx.sudo.sudo(collective.setMembers(members, prime, oldCount));
  }

  const membership = membershipTx(api) as any;
  if (typeof membership.resetMembers === "function") {
    return api.tx.sudo.sudo(membership.resetMembers(members));
  }

  throw new Error("当前 runtime 未找到 reviewCommittee.setMembers 或 reviewMembership.resetMembers。");
}

export function sudoSetPrimeTx(api: ApiPromise, prime: string): SubmittableExtrinsic {
  const membership = membershipTx(api) as any;
  if (typeof membership.setPrime === "function") {
    return api.tx.sudo.sudo(membership.setPrime(prime));
  }

  const collective = committeeTx(api) as any;
  if (typeof collective.setMembers === "function") {
    throw new Error("当前 runtime 使用 collective.setMembers，请在 resetMembers 时同时传 prime。");
  }

  throw new Error("当前 runtime 未找到 reviewMembership.setPrime。");
}

export function setIdentityTx(api: ApiPromise, displayName: string): SubmittableExtrinsic {
  return api.tx.identity.setIdentity({
    display: { Raw: displayName },
    legal: { None: null },
    web: { None: null },
    riot: { None: null },
    email: { None: null },
    pgpFingerprint: null,
    image: { None: null },
    twitter: { None: null },
  });
}

export function sudoAddRegistrarTx(api: ApiPromise, registrarAddress: string): SubmittableExtrinsic {
  return api.tx.sudo.sudo(api.tx.identity.addRegistrar(registrarAddress));
}

export function provideJudgementTx(
  api: ApiPromise,
  registrarIndex: string | number,
  targetAddress: string,
  judgement: "Reasonable" | "KnownGood",
  identityHash: string,
): SubmittableExtrinsic {
  // identity.provideJudgement expects the hash of IdentityInfo, not the full identity object.
  return api.tx.identity.provideJudgement(registrarIndex, targetAddress, judgement, identityHash);
}

export function buildProposal(api: ApiPromise, kind: ProposalKind, value: string | number): SubmittableExtrinsic {
  if (kind === "approve") return api.tx.tasks.approveTask(value);
  if (kind === "reject") return api.tx.tasks.rejectTask(value);
  return api.tx.tasks.setCertificateCollectionId(value);
}

export function committeeProposeTx(
  api: ApiPromise,
  proposal: SubmittableExtrinsic,
  threshold = DEFAULT_COMMITTEE_THRESHOLD,
  lengthBound = DEFAULT_LENGTH_BOUND,
): SubmittableExtrinsic {
  return committeeTx(api).propose(threshold, proposal, lengthBound);
}

export function committeeVoteTx(
  api: ApiPromise,
  proposalHash: string,
  proposalIndex: string | number,
  approve: boolean,
): SubmittableExtrinsic {
  return committeeTx(api).vote(proposalHash, proposalIndex, approve);
}

export function committeeCloseTx(
  api: ApiPromise,
  proposalHash: string,
  proposalIndex: string | number,
  lengthBound = DEFAULT_LENGTH_BOUND,
): SubmittableExtrinsic {
  return committeeTx(api).close(proposalHash, proposalIndex, DEFAULT_WEIGHT_BOUND, lengthBound);
}

export async function readTasks(api: ApiPromise): Promise<TaskView[]> {
  const nextTaskId = Number((await api.query.tasks.nextTaskId()).toString());
  const result: TaskView[] = [];

  for (let id = 0; id < nextTaskId; id += 1) {
    const maybeTask = await api.query.tasks.tasks(id);
    if ((maybeTask as any).isNone) continue;

    const task = (maybeTask as any).unwrap();
    const human = task.toHuman() as Record<string, any>;

    // Keep raw human data for the details drawer; status formatting varies by metadata version.
    result.push({
      id,
      creator: String(human.creator ?? ""),
      assignee: human.assignee ? String(human.assignee) : null,
      reward: String(human.reward ?? ""),
      deadline: String(human.deadline ?? ""),
      certificateCollection: human.certificateCollection ? String(human.certificateCollection) : null,
      status: normalizeStatus(human.status ?? (human as any).Status),
      raw: human,
    });
  }

  return result;
}

export async function readVoting(api: ApiPromise, hash: string): Promise<VotingView> {
  // reviewCommittee.voting(hash) is the source of truth for current ayes/nays/proposal index.
  const voting = await committeeQuery(api).voting(hash);
  if ((voting as any).isNone) {
    return {
      exists: false,
      ayes: [],
      nays: [],
      threshold: "-",
      end: "-",
      index: "-",
      raw: null,
    };
  }

  const human = (voting as any).unwrap().toHuman() as Record<string, any>;
  return {
    exists: true,
    ayes: Array.isArray(human.ayes) ? human.ayes.map(String) : [],
    nays: Array.isArray(human.nays) ? human.nays.map(String) : [],
    threshold: String(human.threshold ?? "-"),
    end: String(human.end ?? "-"),
    index: String(human.index ?? "-"),
    raw: human,
  };
}

export async function readSetupState(api: ApiPromise, bobAddress?: string) {
  const [asset, certificateCollection] = await Promise.all([
    api.query.assets.asset(POINT_ASSET_ID),
    api.query.tasks.certificateCollectionId(),
  ]);
  const certificateCollectionId = certificateCollection.toString();
  const [defaultCollection, activeCollection, members] = await Promise.all([
    api.query.nfts.collection(DEFAULT_CERTIFICATE_COLLECTION_ID),
    api.query.nfts.collection(certificateCollectionId),
    committeeQuery(api).members(),
  ]);
  const prime = await committeeQuery(api).prime();

  const identity = bobAddress ? await api.query.identity.identityOf(bobAddress) : null;

  return {
    assetExists: !(asset as any).isNone,
    defaultCollectionExists: !(defaultCollection as any).isNone,
    activeCollectionExists: !(activeCollection as any).isNone,
    collectionExists: !(activeCollection as any).isNone,
    members: (members.toHuman() as string[]) ?? [],
    prime: prime.toHuman(),
    certificateCollectionId,
    bobIdentity: identity ? identity.toHuman() : null,
    bobIdentityVerified: hasVerifiedJudgement(identity),
  };
}

export async function readIdentityInfoHash(api: ApiPromise, address: string): Promise<string> {
  const identity = await api.query.identity.identityOf(address);
  if ((identity as any).isNone) {
    throw new Error("该账户还没有 identity，请先用 Bob 执行 identity.setIdentity。");
  }

  const registration = (identity as any).unwrap();
  // This matches the Polkadot.js Apps JavaScript helper from the project docs.
  return api.registry.hash(registration.info.toU8a()).toHex();
}

export async function readCertificateOwner(
  api: ApiPromise,
  collectionId: string | number,
  itemId: string | number,
): Promise<string | null> {
  const item = await api.query.nfts.item(collectionId, itemId);
  if ((item as any).isNone) return null;
  const human = (item as any).unwrap().toHuman() as Record<string, any>;
  return human.owner ? String(human.owner) : null;
}

export async function readAssetBalance(api: ApiPromise, address: string): Promise<string> {
  const account = await api.query.assets.account(POINT_ASSET_ID, address);
  if ((account as any).isNone) return "0";
  const human = (account as any).unwrap().toHuman() as Record<string, any>;
  return String(human.balance ?? "0");
}

export function proposalHash(api: ApiPromise, proposal: SubmittableExtrinsic): string {
  return proposal.method.hash.toHex();
}

function defaultNftCollectionConfig(api: ApiPromise) {
  return api.createType("PalletNftsCollectionConfig", {
    settings: 0,
    maxSupply: null,
    mintSettings: {
      mintType: "Issuer",
      price: null,
      startBlock: null,
      endBlock: null,
      defaultItemSettings: 0,
    },
  });
}

function normalizeStatus(status: unknown): string {
  if (typeof status === "string") return status;
  if (status && typeof status === "object") {
    const keys = Object.keys(status as Record<string, unknown>);
    if (keys.length === 1) return keys[0];
    return JSON.stringify(status);
  }
  return String(status);
}

function hasVerifiedJudgement(identity: any): boolean {
  if (!identity || identity.isNone) return false;
  const human = identity.toHuman() as any;
  const judgements = human?.judgements;
  if (!Array.isArray(judgements)) return false;

  return judgements.some((entry) => {
    const value = JSON.stringify(entry);
    return value.includes("Reasonable") || value.includes("KnownGood");
  });
}
