use crate as pallet_counter;

use frame::{
	deps::{
		frame_support::weights::constants::RocksDbWeight,
		frame_system::GenesisConfig,
	},
	prelude::*,
	runtime::prelude::*,
	testing_prelude::*,
};

use polkadot_sdk::pallet_balances;

// Configure a mock runtime to test the pallet.
#[frame_construct_runtime]
mod test_runtime {
	#[runtime::runtime]
	#[runtime::derive(
		RuntimeCall,
		RuntimeEvent,
		RuntimeError,
		RuntimeOrigin,
		RuntimeFreezeReason,
		RuntimeHoldReason,
		RuntimeSlashReason,
		RuntimeLockId,
		RuntimeTask,
		RuntimeViewFunction
	)]
	pub struct Test;

	#[runtime::pallet_index(0)]
	pub type System = frame_system;

	#[runtime::pallet_index(1)]
	pub type Balances = pallet_balances;

	#[runtime::pallet_index(2)]
	pub type Counter = pallet_counter;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Nonce = u64;
	type Block = MockBlock<Test>;
	type BlockHashCount = ConstU64<250>;
	type DbWeight = RocksDbWeight;
	type AccountData = pallet_balances::AccountData<u64>;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub const MaxCounterValue: u32 = 100;
	pub const CounterDeposit: u64 = 10;
}

impl pallet_balances::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = u64;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = ();
	type MaxFreezes = ConstU32<50>;
	type DoneSlashHandler = ();
}

impl pallet_counter::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MaxCounterValue = MaxCounterValue;
	type Currency = Balances;
	type CounterDeposit = CounterDeposit;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> TestState {
	let mut storage = GenesisConfig::<Test>::default().build_storage().unwrap();

	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(1, 100), (2, 100)],
		..Default::default()
	}
	.assimilate_storage(&mut storage)
	.unwrap();

	storage.into()
}