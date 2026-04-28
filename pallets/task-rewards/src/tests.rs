use crate::{
	mock::*,
	Error,
	NextTaskId,
	Scores,
	SubmissionStatus,
	Submissions,
	TaskStatus,
	Tasks,
};

use frame::testing_prelude::*;
use frame::deps::{
	frame_support::{
		storage::unhashed,
		traits::{OnRuntimeUpgrade, StorageVersion},
	},
};


#[test]
fn create_task_works_and_reserves_deposit() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_eq!(NextTaskId::<Test>::get(), 0);
		assert_eq!(Balances::reserved_balance(1), 0);

		assert_ok!(TaskRewards::create_task(
			RuntimeOrigin::signed(1),
			10,
			100,
			2
		));

		assert_eq!(NextTaskId::<Test>::get(), 1);
		assert_eq!(Balances::reserved_balance(1), 10);

		let task = Tasks::<Test>::get(0).expect("task should exist");

		assert_eq!(task.creator, 1);
		assert_eq!(task.reward, 10);
		assert_eq!(task.deposit, 10);
		assert_eq!(task.deadline, 100);
		assert_eq!(task.status, TaskStatus::Open);
		assert_eq!(task.max_submissions, 2);
		assert_eq!(task.submission_count, 0);
	});
}

#[test]
fn create_task_rejects_invalid_reward() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_noop!(
			TaskRewards::create_task(RuntimeOrigin::signed(1), 0, 100, 2),
			Error::<Test>::InvalidReward
		);
	});
}

#[test]
fn create_task_rejects_invalid_deadline() {
	new_test_ext().execute_with(|| {
		System::set_block_number(10);

		assert_noop!(
			TaskRewards::create_task(RuntimeOrigin::signed(1), 10, 10, 2),
			Error::<Test>::InvalidDeadline
		);

		assert_noop!(
			TaskRewards::create_task(RuntimeOrigin::signed(1), 10, 9, 2),
			Error::<Test>::InvalidDeadline
		);
	});
}

#[test]
fn submit_task_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(TaskRewards::create_task(
			RuntimeOrigin::signed(1),
			10,
			100,
			2
		));

		System::set_block_number(5);

		assert_ok!(TaskRewards::submit_task(
			RuntimeOrigin::signed(2),
			0
		));

		let submission = Submissions::<Test>::get(0, 2)
			.expect("submission should exist");

		assert_eq!(submission.submitted_at, 5);
		assert_eq!(submission.status, SubmissionStatus::Pending);
	});
}

#[test]
fn submit_task_rejects_duplicate_submission() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(TaskRewards::create_task(
			RuntimeOrigin::signed(1),
			10,
			100,
			2
		));

		assert_ok!(TaskRewards::submit_task(
			RuntimeOrigin::signed(2),
			0
		));

		assert_noop!(
			TaskRewards::submit_task(RuntimeOrigin::signed(2), 0),
			Error::<Test>::AlreadySubmitted
		);
	});
}

#[test]
fn submit_task_rejects_missing_task() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_noop!(
			TaskRewards::submit_task(RuntimeOrigin::signed(2), 999),
			Error::<Test>::TaskNotFound
		);
	});
}

#[test]
fn submit_task_rejects_after_deadline() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(TaskRewards::create_task(
			RuntimeOrigin::signed(1),
			10,
			5,
			2
		));

		System::set_block_number(6);

		assert_noop!(
			TaskRewards::submit_task(RuntimeOrigin::signed(2), 0),
			Error::<Test>::DeadlinePassed
		);
	});
}

#[test]
fn approve_submission_works_and_adds_score() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(TaskRewards::create_task(
			RuntimeOrigin::signed(1),
			10,
			100,
			2
		));

		assert_ok!(TaskRewards::submit_task(
			RuntimeOrigin::signed(2),
			0
		));

		assert_eq!(Scores::<Test>::get(2), 0);

		assert_ok!(TaskRewards::approve_submission(
			RuntimeOrigin::signed(1),
			0,
			2
		));

		let submission = Submissions::<Test>::get(0, 2)
			.expect("submission should exist");

		assert_eq!(submission.status, SubmissionStatus::Approved);
		assert_eq!(Scores::<Test>::get(2), 10);
	});
}

#[test]
fn reject_submission_works_without_score() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(TaskRewards::create_task(
			RuntimeOrigin::signed(1),
			10,
			100,
			2
		));

		assert_ok!(TaskRewards::submit_task(
			RuntimeOrigin::signed(2),
			0
		));

		assert_ok!(TaskRewards::reject_submission(
			RuntimeOrigin::signed(1),
			0,
			2
		));

		let submission = Submissions::<Test>::get(0, 2)
			.expect("submission should exist");

		assert_eq!(submission.status, SubmissionStatus::Rejected);
		assert_eq!(Scores::<Test>::get(2), 0);
	});
}

#[test]
fn close_task_works_and_unreserves_deposit() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(TaskRewards::create_task(
			RuntimeOrigin::signed(1),
			10,
			100,
			2
		));

		assert_eq!(Balances::reserved_balance(1), 10);

		assert_ok!(TaskRewards::close_task(
			RuntimeOrigin::signed(1),
			0
		));

		let task = Tasks::<Test>::get(0).expect("task should exist");

		assert_eq!(task.status, TaskStatus::Closed);
		assert_eq!(Balances::reserved_balance(1), 0);
	});
}

#[test]
fn close_task_rejects_non_creator() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(TaskRewards::create_task(
			RuntimeOrigin::signed(1),
			10,
			100,
			2
		));

		assert_noop!(
			TaskRewards::close_task(RuntimeOrigin::signed(2), 0),
			Error::<Test>::NotTaskCreator
		);
	});
}

#[test]
fn approve_submission_rejects_non_creator() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(TaskRewards::create_task(
			RuntimeOrigin::signed(1),
			10,
			100,
			2
		));

		assert_ok!(TaskRewards::submit_task(
			RuntimeOrigin::signed(2),
			0
		));

		assert_noop!(
			TaskRewards::approve_submission(RuntimeOrigin::signed(3), 0, 2),
			Error::<Test>::NotTaskCreator
		);
	});
}


#[test]
fn submit_task_rejects_when_max_submissions_reached() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(TaskRewards::create_task(
			RuntimeOrigin::signed(1),
			10,
			100,
			1
		));

		assert_ok!(TaskRewards::submit_task(
			RuntimeOrigin::signed(2),
			0
		));

		assert_noop!(
			TaskRewards::submit_task(RuntimeOrigin::signed(3), 0),
			Error::<Test>::MaxSubmissionsReached
		);

		let task = Tasks::<Test>::get(0).expect("task should exist");
		assert_eq!(task.submission_count, 1);
	});
}


#[test]
fn migration_v1_to_v2_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let old_task: crate::OldTaskInfo<u64, u64, BlockNumberFor<Test>> =
			crate::OldTaskInfo {
				creator: 1u64,
				reward: 10u32,
				deposit: 10u64,
				deadline: 100u64,
				status: TaskStatus::Open,
			};

		let key = Tasks::<Test>::hashed_key_for(0u32);
		unhashed::put(&key, &old_task);

		StorageVersion::new(1).put::<TaskRewards>();

		let _weight =
			<TaskRewards as OnRuntimeUpgrade>::on_runtime_upgrade();

		let task = Tasks::<Test>::get(0u32).expect("task should be migrated");

		assert_eq!(task.creator, 1);
		assert_eq!(task.reward, 10);
		assert_eq!(task.deposit, 10);
		assert_eq!(task.deadline, 100);
		assert_eq!(task.status, TaskStatus::Open);
		assert_eq!(task.max_submissions, 100);
		assert_eq!(task.submission_count, 0);

		assert_eq!(
			TaskRewards::on_chain_storage_version(),
			StorageVersion::new(2)
		);
	});
}
