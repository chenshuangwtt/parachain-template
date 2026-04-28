use crate::{mock::*, Counters};
use frame::testing_prelude::*;
use frame::deps::{
	frame_support::{
		storage::unhashed,
		traits::StorageVersion,
	},
};

#[test]
fn increment_creates_counter_and_reserves_deposit() {
	new_test_ext().execute_with(|| {
		assert_eq!(Counters::<Test>::get(1), None);
		assert_eq!(Balances::reserved_balance(1), 0);

		assert_ok!(Counter::increment(RuntimeOrigin::signed(1)));

		let info = Counters::<Test>::get(1).unwrap();
		assert_eq!(info.value, 1);
		assert_eq!(info.deposit, 10);

		assert_eq!(Balances::reserved_balance(1), 10);
	});
}

#[test]
fn second_increment_does_not_reserve_again() {
	new_test_ext().execute_with(|| {
		assert_ok!(Counter::increment(RuntimeOrigin::signed(1)));
		assert_ok!(Counter::increment(RuntimeOrigin::signed(1)));

		let info = Counters::<Test>::get(1).unwrap();
		assert_eq!(info.value, 2);
		assert_eq!(info.deposit, 10);

		assert_eq!(Balances::reserved_balance(1), 10);
	});
}

#[test]
fn remove_counter_deletes_storage_and_unreserves_deposit() {
	new_test_ext().execute_with(|| {
		assert_ok!(Counter::increment(RuntimeOrigin::signed(1)));

		let info = Counters::<Test>::get(1).unwrap();
		assert_eq!(info.value, 1);
		assert_eq!(info.deposit, 10);
		assert_eq!(Balances::reserved_balance(1), 10);

		assert_ok!(Counter::remove_counter(RuntimeOrigin::signed(1)));

		assert_eq!(Counters::<Test>::get(1), None);
		assert_eq!(Balances::reserved_balance(1), 0);
	});
}

#[test]
fn remove_counter_fails_if_not_exists() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Counter::remove_counter(RuntimeOrigin::signed(1)),
			crate::Error::<Test>::CounterNotFound
		);
	});
}

#[test]
fn migration_v1_to_v2_works() {
	new_test_ext().execute_with(|| {
		let account: u64 = 1;
		let old_value: u32 = 7;

		// 模拟旧版本 storage：Counters<AccountId, u32>
		let key = Counters::<Test>::hashed_key_for(&account);
		unhashed::put(&key, &old_value);

		// 模拟链上当前 pallet storage version 是 v1
		StorageVersion::new(1).put::<Counter>();

		// 执行 migration
		let _weight =
			<Counter as frame::prelude::OnRuntimeUpgrade>::on_runtime_upgrade();

		// 检查迁移后的 storage 已经变成 CounterInfo
		let info = Counters::<Test>::get(account).expect("counter should be migrated");

		assert_eq!(info.value, 7);
		assert_eq!(info.deposit, 10);

		// 检查 storage version 已升级到 v2
		assert_eq!(Counter::on_chain_storage_version(), StorageVersion::new(2));
	});
}