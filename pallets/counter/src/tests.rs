use crate::{mock::*, Counters};
use frame::testing_prelude::*;

#[test]
fn increment_creates_counter_and_reserves_deposit() {
	new_test_ext().execute_with(|| {
		assert_eq!(Counters::<Test>::get(1), None);
		assert_eq!(Balances::reserved_balance(1), 0);

		assert_ok!(Counter::increment(RuntimeOrigin::signed(1)));

		assert_eq!(Counters::<Test>::get(1), Some(1));

		assert_eq!(Balances::reserved_balance(1), 10);
	});
}

#[test]
fn second_increment_does_not_reserve_again() {
	new_test_ext().execute_with(|| {
		assert_ok!(Counter::increment(RuntimeOrigin::signed(1)));
		assert_ok!(Counter::increment(RuntimeOrigin::signed(1)));

		assert_eq!(Counters::<Test>::get(1), Some(2));

		assert_eq!(Balances::reserved_balance(1), 10);
	});
}

#[test]
fn remove_counter_deletes_storage_and_unreserves_deposit() {
	new_test_ext().execute_with(|| {
		assert_ok!(Counter::increment(RuntimeOrigin::signed(1)));

		assert_eq!(Counters::<Test>::get(1), Some(1));
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