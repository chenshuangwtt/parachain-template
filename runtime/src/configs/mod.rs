// This is free and unencumbered software released into the public domain.
//
// Anyone is free to copy, modify, publish, use, compile, sell, or
// distribute this software, either in source code form or as a compiled
// binary, for any purpose, commercial or non-commercial, and by any
// means.
//
// In jurisdictions that recognize copyright laws, the author or authors
// of this software dedicate any and all copyright interest in the
// software to the public domain. We make this dedication for the benefit
// of the public at large and to the detriment of our heirs and
// successors. We intend this dedication to be an overt act of
// relinquishment in perpetuity of all present and future rights to this
// software under copyright law.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
// IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
// OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
// ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
// OTHER DEALINGS IN THE SOFTWARE.
//
// For more information, please refer to <http://unlicense.org>

mod xcm_config;

use polkadot_sdk::{staging_parachain_info as parachain_info, staging_xcm as xcm, *};
#[cfg(not(feature = "runtime-benchmarks"))]
use polkadot_sdk::{staging_xcm_builder as xcm_builder, staging_xcm_executor as xcm_executor};

// Substrate and Polkadot dependencies
use cumulus_pallet_parachain_system::RelayNumberMonotonicallyIncreases;
use cumulus_primitives_core::{AggregateMessageOrigin, ParaId};
use frame_support::{
    derive_impl,
    dispatch::DispatchClass,
    parameter_types,
    traits::{
        ConstBool, ConstU32, ConstU64, ConstU8, EitherOfDiverse, TransformOrigin, VariantCountOf,
    },
    weights::{ConstantMultiplier, Weight},
    PalletId,
};
use frame_system::{
    limits::{BlockLength, BlockWeights},
    EnsureRoot,
};
use pallet_xcm::{EnsureXcm, IsVoiceOfBody};
use parachains_common::message_queue::{NarrowOriginToSibling, ParaIdToSibling};
use polkadot_runtime_common::{
    xcm_sender::ExponentialPrice, BlockHashCount, SlowAdjustingFeeUpdate,
};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_runtime::Perbill;
use sp_version::RuntimeVersion;
use xcm::latest::prelude::{AssetId, BodyId};

// Local module imports
use super::{
    weights::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight},
    AccountId, Assets, Aura, Balance, Balances, Block, BlockNumber, CollatorSelection,
    ConsensusHook, Hash, MessageQueue, Nfts, Nonce, OriginCaller, PalletInfo, ParachainSystem,
    ReviewCommittee, Runtime, RuntimeCall, RuntimeEvent, RuntimeFreezeReason, RuntimeHoldReason,
    RuntimeOrigin, RuntimeTask, Scheduler, Session, SessionKeys, System, WeightToFee, XcmpQueue,
    AVERAGE_ON_INITIALIZE_RATIO, CENTS, EXISTENTIAL_DEPOSIT, HOURS, MAXIMUM_BLOCK_WEIGHT,
    MICRO_UNIT, NORMAL_DISPATCH_RATIO, SLOT_DURATION, UNIT, VERSION,
};
use xcm_config::{RelayLocation, XcmOriginToTransactDispatchOrigin};

parameter_types! {
    pub const Version: RuntimeVersion = VERSION;

    // This part is copied from Substrate's `bin/node/runtime/src/lib.rs`.
    //  The `RuntimeBlockLength` and `RuntimeBlockWeights` exist here because the
    // `DeletionWeightLimit` and `DeletionQueueDepth` depend on those to parameterize
    // the lazy contract deletion.
    pub RuntimeBlockLength: BlockLength =
        BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
        .base_block(BlockExecutionWeight::get())
        .for_class(DispatchClass::all(), |weights| {
            weights.base_extrinsic = ExtrinsicBaseWeight::get();
        })
        .for_class(DispatchClass::Normal, |weights| {
            weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
        })
        .for_class(DispatchClass::Operational, |weights| {
            weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
            // Operational transactions have some extra reserved space, so that they
            // are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
            weights.reserved = Some(
                MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
            );
        })
        .avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
        .build_or_panic();
    pub const SS58Prefix: u16 = 42;
}

/// All migrations of the runtime, aside from the ones declared in the pallets.
///
/// This can be a tuple of types, each implementing `OnRuntimeUpgrade`.
#[allow(unused_parens)]
type SingleBlockMigrations = ();

/// The default types are being injected by [`derive_impl`](`frame_support::derive_impl`) from
/// [`ParaChainDefaultConfig`](`struct@frame_system::config_preludes::ParaChainDefaultConfig`),
/// but overridden as needed.
#[derive_impl(frame_system::config_preludes::ParaChainDefaultConfig)]
impl frame_system::Config for Runtime {
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The index type for storing how many extrinsics an account has signed.
    type Nonce = Nonce;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The block type.
    type Block = Block;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// Runtime version.
    type Version = Version;
    /// The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    /// Block & extrinsics weights: base values and limits.
    type BlockWeights = RuntimeBlockWeights;
    /// The maximum length of a block (in bytes).
    type BlockLength = RuntimeBlockLength;
    /// This is used as an identifier of the chain. 42 is the generic substrate prefix.
    type SS58Prefix = SS58Prefix;
    /// The action to take on a Runtime Upgrade
    type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type SingleBlockMigrations = SingleBlockMigrations;
}

/// Configure the palelt weight reclaim tx.
impl cumulus_pallet_weight_reclaim::Config for Runtime {
    type WeightInfo = ();
}

impl pallet_timestamp::Config for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = Aura;
    type MinimumPeriod = ConstU64<0>;
    type WeightInfo = ();
}

impl pallet_authorship::Config for Runtime {
    type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
    type EventHandler = (CollatorSelection,);
}

parameter_types! {
    pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = ConstU32<50>;
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type FreezeIdentifier = RuntimeFreezeReason;
    type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
    type DoneSlashHandler = ();
}

parameter_types! {
    /// Relay Chain `TransactionByteFee` / 10
    pub const TransactionByteFee: Balance = 10 * MICRO_UNIT;

    pub const TaskDeposit: Balance = UNIT;
}

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = pallet_transaction_payment::FungibleAdapter<Balances, ()>;
    type WeightToFee = WeightToFee;
    type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
    type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
    type OperationalFeeMultiplier = ConstU8<5>;
    type WeightInfo = ();
}

impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type WeightInfo = ();
}

impl pallet_counter::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxCounterValue = MaxCounterValue;
    type Currency = Balances;
    type CounterDeposit = CounterDeposit;
    type WeightInfo = pallet_counter::weights::SubstrateWeight<Runtime>;
}

impl pallet_task_rewards::Config for Runtime {
    type TaskDeposit = TaskDeposit;
    type DefaultMaxSubmissions = DefaultMaxSubmissions;
    type Currency = Balances;
    type WeightInfo = pallet_task_rewards::weights::SubstrateWeight<Runtime>;
}

pub type LocalAssetId = u32;
pub type LocalAssetBalance = u128;

parameter_types! {
    pub const PointAssetId: LocalAssetId = 1;
    pub const DefaultCertificateCollectionId: u32 = 0;
}

pub struct IdentityJudgementVerifier;

impl pallet_tasks::IdentityVerifier<AccountId> for IdentityJudgementVerifier {
    fn is_verified(who: &AccountId) -> bool {
        pallet_identity::IdentityOf::<Runtime>::get(who)
            .map(|registration| {
                registration.judgements.iter().any(|(_, judgement)| {
                    matches!(
                        judgement,
                        pallet_identity::Judgement::Reasonable
                            | pallet_identity::Judgement::KnownGood
                    )
                })
            })
            .unwrap_or(false)
    }
}

impl pallet_tasks::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;

    type Assets = Assets;
    type PointAssetId = PointAssetId;
    type CertificateCollectionId = NftCollectionId;
    type CertificateNfts = Nfts;
    type DefaultCertificateCollectionId = DefaultCertificateCollectionId;
    type CertificateItemConfig = pallet_nfts::ItemConfig;

    type AdminOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, (), 2, 3>;

    type CloseOrigin = frame_system::EnsureRoot<AccountId>;

    type ScheduleOrigin = OriginCaller;
    type Scheduler = Scheduler;
    type TaskRuntimeCall = RuntimeCall;

    type IdentityVerifier = IdentityJudgementVerifier;

    type WeightInfo = pallet_tasks::weights::SubstrateWeight<Runtime>;
}


pub type NftCollectionId = u32;
pub type NftItemId = u32;

parameter_types! {
    pub const NftCollectionDeposit: Balance = UNIT;
    pub const NftItemDeposit: Balance = UNIT;
    pub const NftMetadataDepositBase: Balance = CENTS;
    pub const NftAttributeDepositBase: Balance = CENTS;
    pub const NftDepositPerByte: Balance = MICRO_UNIT;

    pub const NftStringLimit: u32 = 256;
    pub const NftKeyLimit: u32 = 64;
    pub const NftValueLimit: u32 = 256;

    pub const NftApprovalsLimit: u32 = 20;
    pub const NftItemAttributesApprovalsLimit: u32 = 20;
    pub const NftMaxTips: u32 = 10;
    pub const NftMaxDeadlineDuration: BlockNumber = 7 * 24 * HOURS;
    pub const NftMaxAttributesPerCall: u32 = 10;

}

pub struct NftFeatures;

impl frame_support::traits::Get<pallet_nfts::PalletFeatures> for NftFeatures {
    fn get() -> pallet_nfts::PalletFeatures {
        pallet_nfts::PalletFeatures::all_enabled()
    }
}

impl pallet_nfts::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;

    type CollectionId = NftCollectionId;
    type ItemId = NftItemId;

    type Currency = Balances;

    type ForceOrigin = frame_system::EnsureRoot<AccountId>;
    type CreateOrigin = frame_system::EnsureSigned<AccountId>;

    type Locker = ();

    type CollectionDeposit = NftCollectionDeposit;
    type ItemDeposit = NftItemDeposit;
    type MetadataDepositBase = NftMetadataDepositBase;
    type AttributeDepositBase = NftAttributeDepositBase;
    type DepositPerByte = NftDepositPerByte;

    type StringLimit = NftStringLimit;
    type KeyLimit = NftKeyLimit;
    type ValueLimit = NftValueLimit;

    type ApprovalsLimit = NftApprovalsLimit;
    type ItemAttributesApprovalsLimit = NftItemAttributesApprovalsLimit;
    type MaxTips = NftMaxTips;
    type MaxDeadlineDuration = NftMaxDeadlineDuration;
    type MaxAttributesPerCall = NftMaxAttributesPerCall;

    type Features = NftFeatures;

    type OffchainSignature = sp_runtime::MultiSignature;
    type OffchainPublic = sp_runtime::MultiSigner;

    #[cfg(feature = "runtime-benchmarks")]
    type Helper = ();

    type WeightInfo = pallet_nfts::weights::SubstrateWeight<Runtime>;

    type BlockNumberProvider = System;
}


parameter_types! {
    pub const SchedulerMaxScheduledPerBlock: u32 = 50;
}

type BlockNumberProvider = System;

impl pallet_scheduler::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type PalletsOrigin = OriginCaller;

    type RuntimeCall = RuntimeCall;
    type MaximumWeight = ReviewMaxProposalWeight;
    type ScheduleOrigin = frame_system::EnsureRoot<AccountId>;
    type MaxScheduledPerBlock = SchedulerMaxScheduledPerBlock;
    type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
    type OriginPrivilegeCmp = frame_support::traits::EqualPrivilegeOnly;
    type Preimages = ();

    type BlockNumberProvider = System;
}

parameter_types! {
    pub const ReviewMotionDuration: BlockNumber = 5 * HOURS;
    pub const ReviewMaxProposals: u32 = 100;
    pub const ReviewMaxMembers: u32 = 100;
    pub const ReviewMaxProposalWeight: Weight = MAXIMUM_BLOCK_WEIGHT;
}

impl pallet_collective::Config for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;

    type MotionDuration = ReviewMotionDuration;
    type MaxProposals = ReviewMaxProposals;
    type MaxMembers = ReviewMaxMembers;

    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type SetMembersOrigin = frame_system::EnsureRoot<AccountId>;

    type MaxProposalWeight = ReviewMaxProposalWeight;
    type DisapproveOrigin = frame_system::EnsureRoot<AccountId>;
    type KillOrigin = frame_system::EnsureRoot<AccountId>;
    type Consideration = ();

    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

impl pallet_membership::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;

    type AddOrigin = frame_system::EnsureRoot<AccountId>;
    type RemoveOrigin = frame_system::EnsureRoot<AccountId>;
    type SwapOrigin = frame_system::EnsureRoot<AccountId>;
    type ResetOrigin = frame_system::EnsureRoot<AccountId>;
    type PrimeOrigin = frame_system::EnsureRoot<AccountId>;

    type MembershipInitialized = ReviewCommittee;
    type MembershipChanged = ReviewCommittee;

    type MaxMembers = ReviewMaxMembers;

    type WeightInfo = pallet_membership::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const AssetDeposit: Balance = 0;
    pub const AssetAccountDeposit: Balance = 0;
    pub const ApprovalDeposit: Balance = 0;
    pub const StringLimit: u32 = 50;
    pub const MetadataDepositBase: Balance = 0;
    pub const MetadataDepositPerByte: Balance = 0;
    pub const RemoveItemsLimit: u32 = 1000;
}

impl pallet_assets::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;

    type Balance = LocalAssetBalance;
    type AssetId = LocalAssetId;
    type AssetIdParameter = codec::Compact<LocalAssetId>;

    type ReserveData = ();

    type Currency = Balances;

    type CreateOrigin = frame_system::EnsureSigned<AccountId>;
    type ForceOrigin = frame_system::EnsureRoot<AccountId>;

    type AssetDeposit = AssetDeposit;
    type AssetAccountDeposit = AssetAccountDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type ApprovalDeposit = ApprovalDeposit;

    type StringLimit = StringLimit;
    type Freezer = ();
    type Holder = ();
    type Extra = ();
    type CallbackHandle = ();

    type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;

    type RemoveItemsLimit = RemoveItemsLimit;

    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ();
}

parameter_types! {
    pub const BasicDeposit: Balance = UNIT;
    pub const ByteDeposit: Balance = CENTS;
    pub const UsernameDeposit: Balance = UNIT;
    pub const SubAccountDeposit: Balance = UNIT;
    pub const MaxSubAccounts: u32 = 100;
    pub const MaxAdditionalFields: u32 = 10;
    pub const MaxRegistrars: u32 = 20;
    pub const PendingUsernameExpiration: BlockNumber = 7 * 24 * HOURS;
    pub const UsernameGracePeriod: BlockNumber = 7 * 24 * HOURS;
    pub const MaxSuffixLength: u32 = 32;
    pub const MaxUsernameLength: u32 = 64;
}

pub struct UsernameAuthority;

impl frame_support::traits::TypedGet for UsernameAuthority {
    type Type = RuntimeCall;

    fn get() -> Self::Type {
        RuntimeCall::System(frame_system::Call::remark {
            remark: Default::default(),
        })
    }
}

impl pallet_identity::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;

    type BasicDeposit = BasicDeposit;
    type ByteDeposit = ByteDeposit;
    type UsernameDeposit = UsernameDeposit;
    type SubAccountDeposit = SubAccountDeposit;

    type MaxSubAccounts = MaxSubAccounts;
    type IdentityInformation = pallet_identity::legacy::IdentityInfo<MaxAdditionalFields>;
    type MaxRegistrars = MaxRegistrars;

    type Slashed = ();

    type ForceOrigin = frame_system::EnsureRoot<AccountId>;
    type RegistrarOrigin = frame_system::EnsureRoot<AccountId>;
    type UsernameAuthorityOrigin =
        frame_system::EnsureRootWithSuccess<AccountId, UsernameAuthority>;

    type OffchainSignature = sp_runtime::MultiSignature;
    type SigningPublicKey = sp_runtime::MultiSigner;

    type PendingUsernameExpiration = PendingUsernameExpiration;
    type UsernameGracePeriod = UsernameGracePeriod;
    type MaxSuffixLength = MaxSuffixLength;
    type MaxUsernameLength = MaxUsernameLength;

    type WeightInfo = pallet_identity::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
    pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
    pub const RelayOrigin: AggregateMessageOrigin = AggregateMessageOrigin::Parent;
    pub const MaxCounterValue: u32 = 100;
    pub const CounterDeposit: Balance = UNIT;

    pub const DefaultMaxSubmissions: u32 = 100;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
    type WeightInfo = ();
    type RuntimeEvent = RuntimeEvent;
    type OnSystemEvent = ();
    type SelfParaId = parachain_info::Pallet<Runtime>;
    type OutboundXcmpMessageSource = XcmpQueue;
    type DmpQueue = frame_support::traits::EnqueueWithOrigin<MessageQueue, RelayOrigin>;
    type ReservedDmpWeight = ReservedDmpWeight;
    type XcmpMessageHandler = XcmpQueue;
    type ReservedXcmpWeight = ReservedXcmpWeight;
    type CheckAssociatedRelayNumber = RelayNumberMonotonicallyIncreases;
    type ConsensusHook = ConsensusHook;
    type RelayParentOffset = ConstU32<0>;
}

impl parachain_info::Config for Runtime {}

parameter_types! {
    pub MessageQueueServiceWeight: Weight = Perbill::from_percent(35) * RuntimeBlockWeights::get().max_block;
}

impl pallet_message_queue::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    #[cfg(feature = "runtime-benchmarks")]
    type MessageProcessor = pallet_message_queue::mock_helpers::NoopMessageProcessor<
        cumulus_primitives_core::AggregateMessageOrigin,
    >;
    #[cfg(not(feature = "runtime-benchmarks"))]
    type MessageProcessor = xcm_builder::ProcessXcmMessage<
        AggregateMessageOrigin,
        xcm_executor::XcmExecutor<xcm_config::XcmConfig>,
        RuntimeCall,
    >;
    type Size = u32;
    // The XCMP queue pallet is only ever able to handle the `Sibling(ParaId)` origin:
    type QueueChangeHandler = NarrowOriginToSibling<XcmpQueue>;
    type QueuePausedQuery = NarrowOriginToSibling<XcmpQueue>;
    type HeapSize = sp_core::ConstU32<{ 103 * 1024 }>;
    type MaxStale = sp_core::ConstU32<8>;
    type ServiceWeight = MessageQueueServiceWeight;
    type IdleMaxServiceWeight = ();
}

impl cumulus_pallet_aura_ext::Config for Runtime {}

parameter_types! {
    /// The asset ID for the asset that we use to pay for message delivery fees.
    pub FeeAssetId: AssetId = AssetId(xcm_config::RelayLocation::get());
    /// The base fee for the message delivery fees.
    pub const ToSiblingBaseDeliveryFee: u128 = CENTS.saturating_mul(3);
    pub const ToParentBaseDeliveryFee: u128 = CENTS.saturating_mul(3);
}

/// The price for delivering XCM messages to sibling parachains.
pub type PriceForSiblingParachainDelivery =
    ExponentialPrice<FeeAssetId, ToSiblingBaseDeliveryFee, TransactionByteFee, XcmpQueue>;

/// The price for delivering XCM messages to relay chain.
pub type PriceForParentDelivery =
    ExponentialPrice<FeeAssetId, ToParentBaseDeliveryFee, TransactionByteFee, ParachainSystem>;

impl cumulus_pallet_xcmp_queue::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ChannelInfo = ParachainSystem;
    type VersionWrapper = ();
    // Enqueue XCMP messages from siblings for later processing.
    type XcmpQueue = TransformOrigin<MessageQueue, AggregateMessageOrigin, ParaId, ParaIdToSibling>;
    type MaxInboundSuspended = sp_core::ConstU32<1_000>;
    type MaxActiveOutboundChannels = ConstU32<128>;
    type MaxPageSize = ConstU32<{ 1 << 16 }>;
    type ControllerOrigin = EnsureRoot<AccountId>;
    type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
    type WeightInfo = ();
    type PriceForSiblingDelivery = PriceForSiblingParachainDelivery;
}

parameter_types! {
    pub const Period: u32 = 6 * HOURS;
    pub const Offset: u32 = 0;
}

impl pallet_session::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    // we don't have stash and controller, thus we don't need the convert as well.
    type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
    type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
    type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
    type SessionManager = CollatorSelection;
    // Essentially just Aura, but let's be pedantic.
    type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
    type Keys = SessionKeys;
    type DisablingStrategy = ();
    type WeightInfo = ();
    type Currency = Balances;
    type KeyDeposit = ();
}

#[docify::export(aura_config)]
impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
    type DisabledValidators = ();
    type MaxAuthorities = ConstU32<100_000>;
    type AllowMultipleBlocksPerSlot = ConstBool<true>;
    type SlotDuration = ConstU64<SLOT_DURATION>;
}

parameter_types! {
    pub const PotId: PalletId = PalletId(*b"PotStake");
    pub const SessionLength: BlockNumber = 6 * HOURS;
    // StakingAdmin pluralistic body.
    pub const StakingAdminBodyId: BodyId = BodyId::Defense;
}

/// We allow root and the StakingAdmin to execute privileged collator selection operations.
pub type CollatorSelectionUpdateOrigin = EitherOfDiverse<
    EnsureRoot<AccountId>,
    EnsureXcm<IsVoiceOfBody<RelayLocation, StakingAdminBodyId>>,
>;

impl pallet_collator_selection::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type UpdateOrigin = CollatorSelectionUpdateOrigin;
    type PotId = PotId;
    type MaxCandidates = ConstU32<100>;
    type MinEligibleCollators = ConstU32<4>;
    type MaxInvulnerables = ConstU32<20>;
    // should be a multiple of session or things will get inconsistent
    type KickThreshold = Period;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
    type ValidatorRegistration = Session;
    type WeightInfo = ();
}

/// Configure the pallet template in pallets/template.
impl pallet_parachain_template::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_parachain_template::weights::SubstrateWeight<Runtime>;
}
