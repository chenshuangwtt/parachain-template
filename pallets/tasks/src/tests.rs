use crate::{
    mock::{new_test_ext, Assets, RuntimeOrigin, Scheduler, System, Tasks, Test},
    Error, TaskStatus,
};

use frame::deps::{
    frame_support::{assert_noop, assert_ok, traits::OnInitialize},
    sp_runtime::DispatchError,
};

#[test]
fn create_task_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Tasks::create_task(RuntimeOrigin::signed(1), 100, 10));

        let task = Tasks::tasks(0).unwrap();

        assert_eq!(task.creator, 1);
        assert_eq!(task.assignee, None);
        assert_eq!(task.reward, 100);
        assert_eq!(task.deadline, 10);
        assert_eq!(task.status, TaskStatus::Open);
        assert_eq!(Tasks::next_task_id(), 1);
    });
}

#[test]
fn claim_task_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Tasks::create_task(RuntimeOrigin::signed(1), 100, 10));
        assert_ok!(Tasks::claim_task(RuntimeOrigin::signed(2), 0));

        let task = Tasks::tasks(0).unwrap();

        assert_eq!(task.assignee, Some(2));
        assert_eq!(task.status, TaskStatus::Claimed);
    });
}

#[test]
fn submit_task_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Tasks::create_task(RuntimeOrigin::signed(1), 100, 10));
        assert_ok!(Tasks::claim_task(RuntimeOrigin::signed(2), 0));
        assert_ok!(Tasks::submit_task(RuntimeOrigin::signed(2), 0));

        let task = Tasks::tasks(0).unwrap();

        assert_eq!(task.status, TaskStatus::Submitted);
    });
}

#[test]
fn approve_task_mints_points() {
    new_test_ext().execute_with(|| {
        assert_ok!(Tasks::create_task(RuntimeOrigin::signed(1), 100, 10));
        assert_ok!(Tasks::claim_task(RuntimeOrigin::signed(2), 0));
        assert_ok!(Tasks::submit_task(RuntimeOrigin::signed(2), 0));

        assert_eq!(Assets::balance(1, &2), 0);

        assert_ok!(Tasks::approve_task(RuntimeOrigin::root(), 0));

        let task = Tasks::tasks(0).unwrap();

        assert_eq!(task.status, TaskStatus::Approved);
        assert_eq!(Assets::balance(1, &2), 100);
    });
}

#[test]
fn reject_task_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Tasks::create_task(RuntimeOrigin::signed(1), 100, 10));
        assert_ok!(Tasks::claim_task(RuntimeOrigin::signed(2), 0));
        assert_ok!(Tasks::submit_task(RuntimeOrigin::signed(2), 0));

        assert_ok!(Tasks::reject_task(RuntimeOrigin::root(), 0));

        let task = Tasks::tasks(0).unwrap();

        assert_eq!(task.status, TaskStatus::Rejected);
        assert_eq!(Assets::balance(1, &2), 0);
    });
}

#[test]
fn cannot_claim_missing_task() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Tasks::claim_task(RuntimeOrigin::signed(2), 999),
            Error::<Test>::TaskNotFound
        );
    });
}

#[test]
fn cannot_claim_claimed_task() {
    new_test_ext().execute_with(|| {
        assert_ok!(Tasks::create_task(RuntimeOrigin::signed(1), 100, 10));
        assert_ok!(Tasks::claim_task(RuntimeOrigin::signed(2), 0));

        assert_noop!(
            Tasks::claim_task(RuntimeOrigin::signed(2), 0),
            Error::<Test>::TaskNotOpen
        );
    });
}

#[test]
fn cannot_submit_missing_task() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Tasks::submit_task(RuntimeOrigin::signed(2), 999),
            Error::<Test>::TaskNotFound
        );
    });
}

#[test]
fn cannot_submit_unclaimed_task() {
    new_test_ext().execute_with(|| {
        assert_ok!(Tasks::create_task(RuntimeOrigin::signed(1), 100, 10));

        assert_noop!(
            Tasks::submit_task(RuntimeOrigin::signed(2), 0),
            Error::<Test>::TaskNotClaimed
        );
    });
}

#[test]
fn non_assignee_cannot_submit() {
    new_test_ext().execute_with(|| {
        assert_ok!(Tasks::create_task(RuntimeOrigin::signed(1), 100, 10));
        assert_ok!(Tasks::claim_task(RuntimeOrigin::signed(2), 0));

        assert_noop!(
            Tasks::submit_task(RuntimeOrigin::signed(3), 0),
            Error::<Test>::NotAssignee
        );
    });
}

#[test]
fn non_root_cannot_approve() {
    new_test_ext().execute_with(|| {
        assert_ok!(Tasks::create_task(RuntimeOrigin::signed(1), 100, 10));
        assert_ok!(Tasks::claim_task(RuntimeOrigin::signed(2), 0));
        assert_ok!(Tasks::submit_task(RuntimeOrigin::signed(2), 0));

        assert_noop!(
            Tasks::approve_task(RuntimeOrigin::signed(1), 0),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn cannot_approve_before_submit() {
    new_test_ext().execute_with(|| {
        assert_ok!(Tasks::create_task(RuntimeOrigin::signed(1), 100, 10));
        assert_ok!(Tasks::claim_task(RuntimeOrigin::signed(2), 0));

        assert_noop!(
            Tasks::approve_task(RuntimeOrigin::root(), 0),
            Error::<Test>::TaskNotSubmitted
        );
    });
}

#[test]
fn cannot_approve_missing_task() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Tasks::approve_task(RuntimeOrigin::root(), 999),
            Error::<Test>::TaskNotFound
        );
    });
}

#[test]
fn non_root_cannot_reject() {
    new_test_ext().execute_with(|| {
        assert_ok!(Tasks::create_task(RuntimeOrigin::signed(1), 100, 10));
        assert_ok!(Tasks::claim_task(RuntimeOrigin::signed(2), 0));
        assert_ok!(Tasks::submit_task(RuntimeOrigin::signed(2), 0));

        assert_noop!(
            Tasks::reject_task(RuntimeOrigin::signed(1), 0),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn cannot_reject_before_submit() {
    new_test_ext().execute_with(|| {
        assert_ok!(Tasks::create_task(RuntimeOrigin::signed(1), 100, 10));
        assert_ok!(Tasks::claim_task(RuntimeOrigin::signed(2), 0));

        assert_noop!(
            Tasks::reject_task(RuntimeOrigin::root(), 0),
            Error::<Test>::TaskNotSubmitted
        );
    });
}

#[test]
fn unverified_user_cannot_claim_task() {
    new_test_ext().execute_with(|| {
        assert_ok!(Tasks::create_task(RuntimeOrigin::signed(1), 100, 10));

        assert_noop!(
            Tasks::claim_task(RuntimeOrigin::signed(3), 0),
            Error::<Test>::IdentityNotVerified
        );
    });
}

#[test]
fn scheduler_closes_task_at_deadline() {
    new_test_ext().execute_with(|| {
        assert_ok!(Tasks::create_task(RuntimeOrigin::signed(1), 100, 3));

        assert_eq!(Tasks::tasks(0).unwrap().status, TaskStatus::Open);

        System::set_block_number(3);
        Scheduler::on_initialize(3);

        assert_eq!(Tasks::tasks(0).unwrap().status, TaskStatus::Closed);
    });
}
