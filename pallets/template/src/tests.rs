use crate as pallet_template;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, pallet_prelude::*};

#[test]
fn submitting_a_proposal_adds_it_to_queued_proposals() {
	new_test_ext().execute_with(|| {
		assert_ok!(TemplateModule::submit_proposal(
			Origin::signed(1),
			"Should we buy DOT?".encode()
		));
		assert_eq!(TemplateModule::queued_proposals().len(), 1);
		assert_eq!(TemplateModule::queued_proposals()[0], "Should we buy DOT?".encode());
	});
}

#[test]
fn error_if_proposal_too_long() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			TemplateModule::submit_proposal(
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
		assert_ok!(TemplateModule::submit_proposal(Origin::signed(1), "Should we hodl?".encode()));
		assert_noop!(
			TemplateModule::submit_proposal(
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
		assert_eq!(TemplateModule::active_referendum(), None);
		TemplateModule::submit_proposal(Origin::signed(1), "Should we fill the queue?".encode())
			.unwrap();
		run_to_block(<Test as pallet_template::Config>::LaunchPeriod::get());
		assert_eq!(TemplateModule::active_referendum(), Some(()));
	});
}

#[test]
fn referendum_not_started_no_proposals_in_queue() {
	new_test_ext().execute_with(|| {
		assert_eq!(TemplateModule::active_referendum(), None);
		run_to_block(<Test as pallet_template::Config>::LaunchPeriod::get());
		assert_eq!(TemplateModule::active_referendum(), None);
	});
}

#[test]
fn referendum_closes_after_voting_period_blocks() {
	new_test_ext().execute_with(|| {
		assert_eq!(TemplateModule::active_referendum(), None);
		TemplateModule::submit_proposal(Origin::signed(1), "Should we fill the queue?".encode())
			.unwrap();
		run_to_block(LaunchPeriod::get());
		assert_eq!(TemplateModule::active_referendum(), Some(()));
		assert_eq!(TemplateModule::referendum_ends_at(), 3);
		next_block();
		assert_eq!(TemplateModule::active_referendum(), None);
	});
}

#[test]
fn user_submits_vote_and_vote_count_increases() {
	new_test_ext().execute_with(|| {
		assert_ok!(TemplateModule::submit_proposal(
			Origin::signed(1),
			"Should we buy DOT?".encode()
		));
		run_to_block(LaunchPeriod::get());
		assert_eq!(TemplateModule::active_referendum(), Some(()));
		assert_ok!(TemplateModule::submit_vote(
			Origin::signed(1),
			0 as pallet_template::ProposalIndex,
			pallet_template::Vote::Aye
		));
		assert_eq!(TemplateModule::referendum_info(0, 0).unwrap().get_aye_votes(), 1);
		assert_eq!(TemplateModule::referendum_info(0, 0).unwrap().get_nay_votes(), 0);
		assert!(TemplateModule::referendum_info(0, 0).unwrap().is_ongoing());
		next_block();
		assert!(TemplateModule::referendum_info(0, 0).unwrap().has_finished());
	});
}
