use crate as pallet_quadratic_voting;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, pallet_prelude::*};

#[test]
fn submitting_a_proposal_adds_it_to_queued_proposals() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVoting::submit_proposal(
			Origin::signed(1),
			"Should we buy DOT?".encode()
		));
		assert_eq!(QuadraticVoting::queued_proposals().len(), 1);
		assert_eq!(QuadraticVoting::queued_proposals()[0], "Should we buy DOT?".encode());
	});
}

#[test]
fn error_if_proposal_too_long() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			QuadraticVoting::submit_proposal(
				Origin::signed(1),
				"Should we increase the maximum proposal length? Just a little bit".encode()
			),
			Error::<Test>::ProposalTooLong
		);
	});
}

#[test]
fn error_if_queue_full() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVoting::submit_proposal(Origin::signed(1), "Should we hodl?".encode()));
		assert_noop!(
			QuadraticVoting::submit_proposal(
				Origin::signed(1),
				"Should we increase the queue size?".encode()
			),
			Error::<Test>::ProposalQueueFull
		);
	});
}

#[test]
fn referendum_is_started_after_launch_period_blocks() {
	new_test_ext().execute_with(|| {
		assert_eq!(QuadraticVoting::active_referendum(), None);
		QuadraticVoting::submit_proposal(Origin::signed(1), "Should we fill the queue?".encode())
			.unwrap();
		run_to_block(<Test as pallet_quadratic_voting::Config>::LaunchPeriod::get());
		assert_eq!(QuadraticVoting::active_referendum(), Some(()));
	});
}

#[test]
fn referendum_not_started_no_proposals_in_queue() {
	new_test_ext().execute_with(|| {
		assert_eq!(QuadraticVoting::active_referendum(), None);
		run_to_block(<Test as pallet_quadratic_voting::Config>::LaunchPeriod::get());
		assert_eq!(QuadraticVoting::active_referendum(), None);
	});
}

#[test]
fn referendum_closes_after_voting_period_blocks() {
	new_test_ext().execute_with(|| {
		assert_eq!(QuadraticVoting::active_referendum(), None);
		QuadraticVoting::submit_proposal(Origin::signed(1), "Should we fill the queue?".encode())
			.unwrap();
		run_to_block(LaunchPeriod::get());
		assert_eq!(QuadraticVoting::active_referendum(), Some(()));
		assert_eq!(QuadraticVoting::referendum_ends_at(), 3);
		next_block();
		assert_eq!(QuadraticVoting::active_referendum(), None);
	});
}

#[test]
fn user_submits_vote_happy_path() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVoting::submit_proposal(
			Origin::signed(1),
			"Should we buy DOT?".encode()
		));
		run_to_block(LaunchPeriod::get());
		assert_eq!(QuadraticVoting::active_referendum(), Some(()));
		assert_ok!(QuadraticVoting::submit_vote(
			Origin::signed(1),
			0 as pallet_quadratic_voting::ProposalIndex,
			pallet_quadratic_voting::Vote::Aye,
			5
		));
		assert_eq!(QuadraticVoting::referendum_info(0, 0).unwrap().get_aye_votes(), 5);
		assert_eq!(QuadraticVoting::voter_points(1u64).unwrap(), 75u32);
		assert_eq!(QuadraticVoting::referendum_info(0, 0).unwrap().get_nay_votes(), 0);
		assert!(QuadraticVoting::referendum_info(0, 0).unwrap().is_ongoing());
		next_block();
		assert!(QuadraticVoting::referendum_info(0, 0).unwrap().has_finished());
	});
}

#[test]
fn non_voter_should_not_be_allowed_to_submit_anything() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			QuadraticVoting::submit_proposal(Origin::signed(2), "Will you let me in?".encode()),
			Error::<Test>::NotAVoter
		);
		assert_ok!(QuadraticVoting::submit_proposal(Origin::signed(1), "Yeah, right".encode()));
		run_to_block(LaunchPeriod::get());
		assert_noop!(
			QuadraticVoting::submit_vote(
				Origin::signed(2),
				0 as pallet_quadratic_voting::ProposalIndex,
				pallet_quadratic_voting::Vote::Nay,
				1
			),
			Error::<Test>::NotAVoter
		);
	});
}

#[test]
fn try_vote_with_not_enough_points() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVoting::submit_proposal(Origin::signed(1), "Let's go".encode()));
		run_to_block(LaunchPeriod::get());
		assert_noop!(
			QuadraticVoting::submit_vote(
				Origin::signed(1),
				0 as pallet_quadratic_voting::ProposalIndex,
				pallet_quadratic_voting::Vote::Nay,
				11
			),
			Error::<Test>::NotEnoughPoints
		);
	});
}

#[test]
fn double_voting_fails() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVoting::submit_proposal(Origin::signed(1), "Let's go".encode()));
		run_to_block(LaunchPeriod::get());
		assert_ok!(QuadraticVoting::submit_vote(
			Origin::signed(1),
			0 as pallet_quadratic_voting::ProposalIndex,
			pallet_quadratic_voting::Vote::Nay,
			5
		));
		assert_noop!(
			QuadraticVoting::submit_vote(
				Origin::signed(1),
				0 as pallet_quadratic_voting::ProposalIndex,
				pallet_quadratic_voting::Vote::Nay,
				5
			),
			Error::<Test>::AlreadyVoted
		);
	});
}
