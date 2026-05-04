use crate as pallet_tasks;

use frame::deps::{
    frame_support::{
        assert_ok, construct_runtime, derive_impl, parameter_types,
        traits::{AsEnsureOriginWithArg, ConstU32},
    },
    frame_system,
    sp_core::H256,
    sp_io,
    sp_runtime::{
        testing::{TestSignature, UintAuthorityId},
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    },
};

use polkadot_sdk::{pallet_assets, pallet_balances, pallet_nfts, pallet_scheduler};

pub type AccountId = u64;
pub type Balance = u128;
pub type LocalAssetId = u32;
pub type LocalAssetBalance = u128;

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        Assets: pallet_assets,
        Nfts: pallet_nfts,
        Scheduler: pallet_scheduler,
        Tasks: pallet_tasks,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub RuntimeBlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(
            frame::deps::frame_support::weights::Weight::from_parts(2_000_000_000_000, u64::MAX),
        );
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type BlockHashCount = BlockHashCount;
    type AccountData = pallet_balances::AccountData<Balance>;
    type BlockWeights = RuntimeBlockWeights;
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 1;
}

impl pallet_balances::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ConstU32<0>;
    type DoneSlashHandler = ();
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

pub struct TestIdentityVerifier;

impl pallet_tasks::IdentityVerifier<AccountId> for TestIdentityVerifier {
    fn is_verified(who: &AccountId) -> bool {
        *who == 2
    }
}

impl pallet_assets::Config for Test {
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

    type RemoveItemsLimit = RemoveItemsLimit;

    type WeightInfo = ();

    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ();
}

pub type NftCollectionId = u32;
pub type NftItemId = u32;

parameter_types! {
    pub const NftCollectionDeposit: Balance = 0;
    pub const NftItemDeposit: Balance = 0;
    pub const NftMetadataDepositBase: Balance = 0;
    pub const NftAttributeDepositBase: Balance = 0;
    pub const NftDepositPerByte: Balance = 0;
    pub const NftStringLimit: u32 = 256;
    pub const NftKeyLimit: u32 = 64;
    pub const NftValueLimit: u32 = 256;
    pub const NftApprovalsLimit: u32 = 20;
    pub const NftItemAttributesApprovalsLimit: u32 = 20;
    pub const NftMaxTips: u32 = 10;
    pub const NftMaxDeadlineDuration: u64 = 100;
    pub const NftMaxAttributesPerCall: u32 = 10;
    pub storage NftFeatures: pallet_nfts::PalletFeatures =
        pallet_nfts::PalletFeatures::all_enabled();
}

impl pallet_nfts::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type CollectionId = NftCollectionId;
    type ItemId = NftItemId;
    type Currency = Balances;
    type ForceOrigin = frame_system::EnsureRoot<AccountId>;
    type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
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
    type OffchainSignature = TestSignature;
    type OffchainPublic = UintAuthorityId;
    type WeightInfo = ();
    #[cfg(feature = "runtime-benchmarks")]
    type Helper = ();
    type BlockNumberProvider = System;
}

parameter_types! {
    pub const SchedulerMaxScheduledPerBlock: u32 = 50;
    pub const SchedulerMaxWeight: frame::deps::frame_support::weights::Weight =
        frame::deps::frame_support::weights::Weight::from_parts(1_600_000_000_000, u64::MAX);
}

impl pallet_scheduler::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type PalletsOrigin = OriginCaller;

    type RuntimeCall = RuntimeCall;
    type MaximumWeight = SchedulerMaxWeight;
    type ScheduleOrigin = frame_system::EnsureRoot<AccountId>;
    type MaxScheduledPerBlock = SchedulerMaxScheduledPerBlock;
    type WeightInfo = ();
    type OriginPrivilegeCmp = frame::deps::frame_support::traits::EqualPrivilegeOnly;
    type Preimages = ();
    type BlockNumberProvider = System;
}

parameter_types! {
    pub const PointAssetId: LocalAssetId = 1;
    pub const DefaultCertificateCollectionId: NftCollectionId = 0;
}

impl pallet_tasks::Config for Test {
    type RuntimeEvent = RuntimeEvent;

    type Assets = Assets;
    type PointAssetId = PointAssetId;
    type CertificateCollectionId = NftCollectionId;
    type CertificateNfts = Nfts;
    type DefaultCertificateCollectionId = DefaultCertificateCollectionId;
    type CertificateItemConfig = pallet_nfts::ItemConfig;

    type AdminOrigin = frame_system::EnsureRoot<AccountId>;

    type CloseOrigin = frame_system::EnsureRoot<AccountId>;

    type ScheduleOrigin = OriginCaller;
    type Scheduler = Scheduler;
    type TaskRuntimeCall = RuntimeCall;

    type IdentityVerifier = TestIdentityVerifier;

    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    new_test_ext_with_certificate_collection(true)
}

pub fn new_test_ext_without_certificate_collection() -> sp_io::TestExternalities {
    new_test_ext_with_certificate_collection(false)
}

fn new_test_ext_with_certificate_collection(
    create_certificate_collection: bool,
) -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 1_000_000), (2, 1_000_000), (3, 1_000_000)],
        dev_accounts: Default::default(),
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(storage);

    ext.execute_with(|| {
        System::set_block_number(1);

        assert_ok!(Assets::force_create(
            RuntimeOrigin::root(),
            codec::Compact(1),
            1,
            true,
            1,
        ));

        if create_certificate_collection {
            assert_ok!(Nfts::force_create(
                RuntimeOrigin::root(),
                1,
                Default::default(),
            ));
        }
    });

    ext
}
