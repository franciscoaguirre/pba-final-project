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
fn referenda_is_created_after_launch_period_blocks() {
	new_test_ext().execute_with(|| {
		assert_eq!(TemplateModule::active_referendum(), None);
		run_to_block(<Test as pallet_template::Config>::LaunchPeriod::get());
		assert_eq!(TemplateModule::active_referendum(), Some(()));
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
