#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;


#[frame::pallet]
pub mod pallet {
    use frame::prelude::*;
    use frame::deps::frame_system;
    use frame::deps::frame_support::traits::{Currency, ReservableCurrency};
    use crate::weights::WeightInfo;

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[derive(
        Encode,
        Decode,
        Clone,
        Eq,
        PartialEq,
        RuntimeDebug,
        TypeInfo,
        MaxEncodedLen,
    )]
    pub struct CounterInfo<Balance> {
        pub value: u32,
        pub deposit: Balance,
    }


    #[pallet::config]
    pub trait Config: frame_system::Config<RuntimeEvent: From<Event<Self>>>  {
        type RuntimeEvent: From<Event<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        // 定义一个常量，表示计数器的最大值    
        #[pallet::constant]
        type MaxCounterValue: Get<u32>;   

        type Currency: ReservableCurrency<Self::AccountId>;

        #[pallet::constant]
        type CounterDeposit: Get<BalanceOf<Self>>;

        type WeightInfo: crate::weights::WeightInfo;
    }

    // pub trait WeightInfo {
    //     fn increment() -> Weight;
    //     fn set_value() -> Weight;
    //     fn remove_counter() -> Weight;
    // }

    // impl WeightInfo for () {
    //     fn increment() -> Weight {
    //         Weight::from_parts(10_000, 0)
    //     }

    //     fn set_value() -> Weight {
    //         Weight::from_parts(10_000, 0)
    //     }

    //     fn remove_counter() -> Weight {
    //         Weight::from_parts(10_000, 0)
    //     }
    // }

    pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;


    #[pallet::storage]
    #[pallet::getter(fn counters)]
    pub type Counters<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        <T as frame_system::Config>::AccountId,
        CounterInfo<BalanceOf<T>>,
        OptionQuery,
    >;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            let onchain_version = Pallet::<T>::on_chain_storage_version();

            if onchain_version == StorageVersion::new(1) {
                let mut migrated: u64 = 0;

                Counters::<T>::translate::<u32, _>(
                    |_account: <T as frame_system::Config>::AccountId, old_value: u32| {
                        migrated = migrated.saturating_add(1);

                        Some(CounterInfo {
                            value: old_value,
                            deposit: T::CounterDeposit::get(),
                        })
                    }
                );

                STORAGE_VERSION.put::<Pallet<T>>();

                T::DbWeight::get().reads_writes(migrated + 1, migrated + 1)
            } else {
                T::DbWeight::get().reads(1)
            }
        }
    }


    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Incremented {
            who: <T as frame_system::Config>::AccountId,
            value: u32,
        },
        ValueSet {
            who: <T as frame_system::Config>::AccountId,
            value: u32,
        },
        CounterRemoved {
            who: <T as frame_system::Config>::AccountId,
            value: u32,
            deposit: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        Overflow,
        CounterTooLarge,
        CounterNotFound,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::increment())]
        pub fn increment(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let info = match Counters::<T>::get(&who) {
                Some(mut info) => {
                    let new_value = info
                        .value
                        .checked_add(1)
                        .ok_or(Error::<T>::Overflow)?;

                    ensure!(
                        new_value <= T::MaxCounterValue::get(),
                        Error::<T>::CounterTooLarge
                    );

                    info.value = new_value;
                    info
                },
                None => {
                    let deposit = T::CounterDeposit::get();

                    T::Currency::reserve(&who, deposit)?;

                    CounterInfo {
                        value: 1,
                        deposit,
                    }
                },
            };

            Counters::<T>::insert(&who, &info);

            Self::deposit_event(Event::Incremented {
                who,
                value: info.value,
            });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::set_value())]
        pub fn set_value(origin: OriginFor<T>,  who: T::AccountId, value: u32) -> DispatchResult {
            ensure_root(origin)?;

            ensure!(
                value <= T::MaxCounterValue::get(),
                Error::<T>::CounterTooLarge
            );

            let info = match Counters::<T>::get(&who) {
                Some(mut info) => {
                    info.value = value;
                    info
                },
                None => {
                    let deposit = T::CounterDeposit::get();

                    T::Currency::reserve(&who, deposit)?;

                    CounterInfo {
                        value,
                        deposit,
                    }
                },
            };

            Counters::<T>::insert(&who, &info);

            Self::deposit_event(Event::ValueSet {
                who,
                value,
            });

            Ok(())
        }


        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::remove_counter())]
        pub fn remove_counter(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let info = Counters::<T>::get(&who)
                .ok_or(Error::<T>::CounterNotFound)?;

            Counters::<T>::remove(&who);

            T::Currency::unreserve(&who, info.deposit);

            Self::deposit_event(Event::CounterRemoved {
                who,
                value: info.value,
                deposit: info.deposit,
            });
            Ok(())
        }
    }

}