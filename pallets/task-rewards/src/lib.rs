#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame::pallet]
pub mod pallet {
	use frame::deps::frame_support::traits::{Currency, ReservableCurrency};
	use frame::prelude::*;
    use frame_system::pallet_prelude::BlockNumberFor;

	use crate::weights::WeightInfo;

	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;


	pub type TaskId = u32;

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
	pub enum TaskStatus {
		Open,
		Closed,
	}

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
	pub struct TaskInfo<AccountId, Balance, BlockNumber> {
		pub creator: AccountId,
		pub reward: u32,
		pub deposit: Balance,
		pub deadline: BlockNumber,
		pub status: TaskStatus,
	}

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
    pub enum SubmissionStatus {
        Pending,
        Approved,
        Rejected,
    }

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
    pub struct SubmissionInfo<BlockNumber> {
        pub submitted_at: BlockNumber,
        pub status: SubmissionStatus,
    }    

	#[pallet::config]
	pub trait Config: frame_system::Config<RuntimeEvent: From<Event<Self>>> {
		#[pallet::constant]
		type TaskDeposit: Get<BalanceOf<Self>>;

		type Currency: ReservableCurrency<<Self as frame_system::Config>::AccountId>;

		type WeightInfo: crate::weights::WeightInfo;
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn next_task_id)]
	pub type NextTaskId<T> = StorageValue<_, TaskId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tasks)]
	pub type Tasks<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		TaskId,
		TaskInfo<
            <T as frame_system::Config>::AccountId,
            BalanceOf<T>,
            BlockNumberFor<T>,
        >,
		OptionQuery,
	>;

    #[pallet::storage]
    #[pallet::getter(fn submissions)]
    pub type Submissions<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        TaskId,
        Blake2_128Concat,
        <T as frame_system::Config>::AccountId,
        SubmissionInfo<BlockNumberFor<T>>,
        OptionQuery,
    >;

	#[pallet::storage]
	#[pallet::getter(fn scores)]
	pub type Scores<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		<T as frame_system::Config>::AccountId,
		u32,
		ValueQuery,
	>;


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TaskCreated {
			task_id: TaskId,
			creator: <T as frame_system::Config>::AccountId,
			reward: u32,
			deposit: BalanceOf<T>,
			deadline: BlockNumberFor<T>,
		},

        TaskSubmitted {
            task_id: TaskId,
            who: <T as frame_system::Config>::AccountId,
            submitted_at: BlockNumberFor<T>,
        },

		SubmissionApproved {
			task_id: TaskId,
			who: <T as frame_system::Config>::AccountId,
			reward: u32,
			new_score: u32,
		},

		SubmissionRejected {
			task_id: TaskId,
			who: <T as frame_system::Config>::AccountId,
		},

		TaskClosed {
			task_id: TaskId,
			creator: <T as frame_system::Config>::AccountId,
			deposit: BalanceOf<T>,
		},

	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidReward,
		InvalidDeadline,
		TaskIdOverflow,
        TaskNotFound,
        TaskClosed,
        DeadlinePassed,
        AlreadySubmitted,
		SubmissionNotFound,
		SubmissionNotPending,
		ScoreOverflow,
		NotTaskCreator,
		TaskAlreadyClosed,
		NotReviewer
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create_task())]
		pub fn create_task(
			origin: OriginFor<T>,
			reward: u32,
			deadline: BlockNumberFor<T>,
		) -> DispatchResult {
			let creator = ensure_signed(origin)?;

			ensure!(reward > 0, Error::<T>::InvalidReward);

			let current_block: BlockNumberFor<T> = frame_system::Pallet::<T>::block_number();
            ensure!(deadline > current_block, Error::<T>::InvalidDeadline);

			let task_id = NextTaskId::<T>::get();
			let next_task_id = task_id
				.checked_add(1)
				.ok_or(Error::<T>::TaskIdOverflow)?;

			let deposit = T::TaskDeposit::get();
			T::Currency::reserve(&creator, deposit)?;

			let task = TaskInfo {
				creator: creator.clone(),
				reward,
				deposit,
				deadline,
				status: TaskStatus::Open,
			};

			Tasks::<T>::insert(task_id, task);
			NextTaskId::<T>::put(next_task_id);

			Self::deposit_event(Event::TaskCreated {
				task_id,
				creator,
				reward,
				deposit,
				deadline,
			});

			Ok(())
		}


        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::submit_task())]
        pub fn submit_task(
            origin: OriginFor<T>,
            task_id: TaskId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let task = Tasks::<T>::get(task_id)
                .ok_or(Error::<T>::TaskNotFound)?;

            ensure!(
                task.status == TaskStatus::Open,
                Error::<T>::TaskClosed
            );

            let current_block: BlockNumberFor<T> =
                frame_system::Pallet::<T>::block_number();

            ensure!(
                current_block <= task.deadline,
                Error::<T>::DeadlinePassed
            );

            ensure!(
                !Submissions::<T>::contains_key(task_id, &who),
                Error::<T>::AlreadySubmitted
            );

            let submission = SubmissionInfo {
                submitted_at: current_block,
                status: SubmissionStatus::Pending,
            };

            Submissions::<T>::insert(task_id, &who, submission);

            Self::deposit_event(Event::TaskSubmitted {
                task_id,
                who,
                submitted_at: current_block,
            });

            Ok(())
        }

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::approve_submission())]
		pub fn approve_submission(
			origin: OriginFor<T>,
			task_id: TaskId,
			who: <T as frame_system::Config>::AccountId,
		) -> DispatchResult {
			// ensure_root(origin)?;
			let caller = ensure_signed(origin)?;

			let task = Tasks::<T>::get(task_id)
				.ok_or(Error::<T>::TaskNotFound)?;

			ensure!(
				caller == task.creator,
				Error::<T>::NotTaskCreator
			);		

			let mut submission = Submissions::<T>::get(task_id, &who)
				.ok_or(Error::<T>::SubmissionNotFound)?;

			ensure!(
				submission.status == SubmissionStatus::Pending,
				Error::<T>::SubmissionNotPending
			);

			let old_score = Scores::<T>::get(&who);
			let new_score = old_score
				.checked_add(task.reward)
				.ok_or(Error::<T>::ScoreOverflow)?;

			submission.status = SubmissionStatus::Approved;

			Submissions::<T>::insert(task_id, &who, submission);
			Scores::<T>::insert(&who, new_score);

			Self::deposit_event(Event::SubmissionApproved {
				task_id,
				who,
				reward: task.reward,
				new_score,
			});

			Ok(())
		}


		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::reject_submission())]
		pub fn reject_submission(
			origin: OriginFor<T>,
			task_id: TaskId,
			who: <T as frame_system::Config>::AccountId,
		) -> DispatchResult {
			// ensure_root(origin)?;
			let caller = ensure_signed(origin)?;

			let task = Tasks::<T>::get(task_id)
				.ok_or(Error::<T>::TaskNotFound)?;

			ensure!(
				caller == task.creator,
				Error::<T>::NotTaskCreator
			);

			let mut submission = Submissions::<T>::get(task_id, &who)
				.ok_or(Error::<T>::SubmissionNotFound)?;

			ensure!(
				submission.status == SubmissionStatus::Pending,
				Error::<T>::SubmissionNotPending
			);

			submission.status = SubmissionStatus::Rejected;

			Submissions::<T>::insert(task_id, &who, submission);

			Self::deposit_event(Event::SubmissionRejected {
				task_id,
				who,
			});

			Ok(())
		}



		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::close_task())]
		pub fn close_task(
			origin: OriginFor<T>,
			task_id: TaskId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Tasks::<T>::try_mutate(task_id, |maybe_task| -> DispatchResult {
				let task = maybe_task
					.as_mut()
					.ok_or(Error::<T>::TaskNotFound)?;

				ensure!(
					task.creator == who,
					Error::<T>::NotTaskCreator
				);

				ensure!(
					task.status == TaskStatus::Open,
					Error::<T>::TaskAlreadyClosed
				);

				task.status = TaskStatus::Closed;

				T::Currency::unreserve(&who, task.deposit);

				Self::deposit_event(Event::TaskClosed {
					task_id,
					creator: who.clone(),
					deposit: task.deposit,
				});

				Ok(())
			})
		}

	}
}