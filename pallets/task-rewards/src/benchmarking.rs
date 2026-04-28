//! Benchmarking setup for pallet-task-rewards

use super::*;
use frame::{deps::frame_benchmarking::v2::*, prelude::*};

#[benchmarks]
mod benchmarks {
	use super::*;

	#[cfg(test)]
	use crate::pallet::Pallet as TaskRewards;

    use frame::deps::frame_support::traits::{Currency, ReservableCurrency};
	use frame_system::RawOrigin;

	fn fund_account<T: Config>(who: &T::AccountId) {
		T::Currency::make_free_balance_be(
			who,
			T::TaskDeposit::get() * 10u32.into(),
		);
	}

	fn create_open_task<T: Config>(
		creator: &T::AccountId,
		reward: u32,
		deadline: BlockNumberFor<T>,
		max_submissions: u32,
	) -> TaskId {
		let task_id = NextTaskId::<T>::get();

		let deposit = T::TaskDeposit::get();

		T::Currency::reserve(creator, deposit)
			.expect("creator should have enough balance");

		let task = TaskInfo {
			creator: creator.clone(),
			reward,
			deposit,
			deadline,
			status: TaskStatus::Open,
			max_submissions,
			submission_count: 0,
		};

		Tasks::<T>::insert(task_id, task);
		NextTaskId::<T>::put(task_id + 1);

		task_id
	}

	fn create_pending_submission<T: Config>(
		task_id: TaskId,
		who: &T::AccountId,
		submitted_at: BlockNumberFor<T>,
	) {
		let submission = SubmissionInfo {
			submitted_at,
			status: SubmissionStatus::Pending,
		};

		Submissions::<T>::insert(task_id, who, submission);
	}

	#[benchmark]
	fn create_task() {
		let caller: T::AccountId = whitelisted_caller();
		fund_account::<T>(&caller);

		let reward: u32 = 10;
		let current_block: BlockNumberFor<T> = 1u32.into();
        frame_system::Pallet::<T>::set_block_number(current_block);

        let deadline = current_block + 100u32.into();

		let max_submissions: u32 = 100;

		#[extrinsic_call]
		create_task(RawOrigin::Signed(caller.clone()), reward, deadline,max_submissions);

		let task = Tasks::<T>::get(0).expect("task should exist");

		assert_eq!(NextTaskId::<T>::get(), 1);
		assert_eq!(task.creator, caller);
		assert_eq!(task.reward, reward);
		assert_eq!(task.deposit, T::TaskDeposit::get());
		assert_eq!(task.deadline, deadline);
		assert_eq!(task.status, TaskStatus::Open);
		assert_eq!(task.max_submissions, max_submissions);
		assert_eq!(task.submission_count, 0);
	}

	#[benchmark]
	fn submit_task() {
		let creator: T::AccountId = whitelisted_caller();
		let submitter: T::AccountId = account("submitter", 0, 0);

		fund_account::<T>(&creator);
		fund_account::<T>(&submitter);

		let current_block: BlockNumberFor<T> = 1u32.into();
        frame_system::Pallet::<T>::set_block_number(current_block);

        let deadline = current_block + 100u32.into();
		
		let task_id = create_open_task::<T>(&creator, 10, deadline, 100);

		#[extrinsic_call]
		submit_task(RawOrigin::Signed(submitter.clone()), task_id);

		let submission = Submissions::<T>::get(task_id, &submitter)
			.expect("submission should exist");

		assert_eq!(submission.submitted_at, current_block);
		assert_eq!(submission.status, SubmissionStatus::Pending);

		let task = Tasks::<T>::get(task_id).expect("task should exist");
		assert_eq!(task.submission_count, 1);
	}

	#[benchmark]
	fn approve_submission() {
		let creator: T::AccountId = whitelisted_caller();
		let submitter: T::AccountId = account("submitter", 0, 0);

		fund_account::<T>(&creator);
		fund_account::<T>(&submitter);

		let current_block: BlockNumberFor<T> = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(current_block);

		let deadline = current_block + 100u32.into();

		let reward: u32 = 10;
		let task_id = create_open_task::<T>(&creator, 10, deadline, 100);
		create_pending_submission::<T>(task_id, &submitter, current_block);

		#[extrinsic_call]
		approve_submission(
			RawOrigin::Signed(creator.clone()),
			task_id,
			submitter.clone()
		);


		let submission = Submissions::<T>::get(task_id, &submitter)
			.expect("submission should exist");

		assert_eq!(submission.status, SubmissionStatus::Approved);
		assert_eq!(Scores::<T>::get(&submitter), reward);
	}


	#[benchmark]
	fn reject_submission() {
		let creator: T::AccountId = whitelisted_caller();
		let submitter: T::AccountId = account("submitter", 0, 0);

		fund_account::<T>(&creator);
		fund_account::<T>(&submitter);

		let current_block: BlockNumberFor<T> = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(current_block);

		let deadline = current_block + 100u32.into();

		let task_id = create_open_task::<T>(&creator, 10, deadline, 100);
		create_pending_submission::<T>(task_id, &submitter, current_block);

		#[extrinsic_call]
		reject_submission(
			RawOrigin::Signed(creator.clone()),
			task_id,
			submitter.clone()
		);


		let submission = Submissions::<T>::get(task_id, &submitter)
			.expect("submission should exist");

		assert_eq!(submission.status, SubmissionStatus::Rejected);
		assert_eq!(Scores::<T>::get(&submitter), 0);
	}

	#[benchmark]
	fn close_task() {
		let creator: T::AccountId = whitelisted_caller();
		fund_account::<T>(&creator);

		let current_block: BlockNumberFor<T> = 1u32.into();
        frame_system::Pallet::<T>::set_block_number(current_block);
        let deadline = current_block + 100u32.into();

		let task_id = create_open_task::<T>(&creator, 10, deadline, 100);

		#[extrinsic_call]
		close_task(RawOrigin::Signed(creator.clone()), task_id);

		let task = Tasks::<T>::get(task_id).expect("task should exist");

		assert_eq!(task.status, TaskStatus::Closed);
		assert_eq!(T::Currency::reserved_balance(&creator), 0u32.into());
	}

	impl_benchmark_test_suite!(
		TaskRewards,
		crate::mock::new_test_ext(),
		crate::mock::Test
	);
}