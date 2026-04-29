use crate as pallet_tasks;

use frame::{
	deps::{
		frame_support::{
			construct_runtime, derive_impl, parameter_types,
			traits::{ConstU32, ConstU64},
		},
		frame_system,
		sp_core::H256,
		sp_io,
		sp_runtime::{
			traits::{BlakeTwo256, IdentityLookup},
			BuildStorage,
		},
	},
	prelude::*,
};


use polkadot_sdk::{pallet_assets, pallet_balances};

pub type AccountId = u64;
pub type Balance = u128;
pub type LocalAssetId = u32;
pub type LocalAssetBalance = u128;

pub type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		Assets: pallet_assets,
		Tasks: pallet_tasks,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
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

parameter_types! {
	pub const PointAssetId: LocalAssetId = 1;
}

impl pallet_tasks::Config for Test {
	type RuntimeEvent = RuntimeEvent;

	type Assets = Assets;
	type PointAssetId = PointAssetId;

	type AdminOrigin = frame_system::EnsureRoot<AccountId>;

	type IdentityVerifier = TestIdentityVerifier;

	type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap();

	pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 1_000_000),
            (2, 1_000_000),
            (3, 1_000_000),
        ],
        dev_accounts: Default::default(),
    }
    .assimilate_storage(&mut storage)
    .unwrap();

	let mut ext = sp_io::TestExternalities::new(storage);

	ext.execute_with(|| {
		System::set_block_number(1);

		pallet_assets::Pallet::<Test>::force_create(
			RuntimeOrigin::root(),
			codec::Compact(1),
			1,
			true,
			1,
		)
		.unwrap();
	});

	ext
}