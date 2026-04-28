//! Benchmarking setup for pallet-counter

use super::*;
use frame::{deps::frame_benchmarking::v2::*, prelude::*};

#[benchmarks]
mod benchmarks {
	use super::*;

	#[cfg(test)]
	use crate::pallet::Pallet as Counter;

	use frame::deps::frame_support::traits::Currency;
	use frame_system::RawOrigin;

	#[benchmark]
	fn increment() {
		let caller: T::AccountId = whitelisted_caller();

		T::Currency::make_free_balance_be(
			&caller,
			T::CounterDeposit::get() * 10u32.into(),
		);

		#[extrinsic_call]
		increment(RawOrigin::Signed(caller.clone()));

		let info = Counters::<T>::get(&caller).expect("counter should exist");
		assert_eq!(info.value, 1);
		assert_eq!(info.deposit, T::CounterDeposit::get());
	}

	#[benchmark]
	fn set_value() {
		let caller: T::AccountId = whitelisted_caller();
		let value: u32 = 10;

		T::Currency::make_free_balance_be(
			&caller,
			T::CounterDeposit::get() * 10u32.into(),
		);

		#[extrinsic_call]
		set_value(RawOrigin::Root, caller.clone(), value);

		let info = Counters::<T>::get(&caller).expect("counter should exist");
		assert_eq!(info.value, value);
		assert_eq!(info.deposit, T::CounterDeposit::get());
	}

	#[benchmark]
	fn remove_counter() {
		let caller: T::AccountId = whitelisted_caller();

		T::Currency::make_free_balance_be(
			&caller,
			T::CounterDeposit::get() * 10u32.into(),
		);

		Pallet::<T>::increment(RawOrigin::Signed(caller.clone()).into())
			.expect("increment should work");

		let info = Counters::<T>::get(&caller).expect("counter should exist");
		assert_eq!(info.value, 1);
		assert_eq!(info.deposit, T::CounterDeposit::get());

		#[extrinsic_call]
		remove_counter(RawOrigin::Signed(caller.clone()));

		assert_eq!(Counters::<T>::get(&caller), None);
	}

	impl_benchmark_test_suite!(Counter, crate::mock::new_test_ext(), crate::mock::Test);
}