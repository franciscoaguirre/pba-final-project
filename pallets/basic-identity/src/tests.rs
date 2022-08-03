use sp_core::H256;

use crate::mock::*;
use frame_support::assert_ok;

#[test]
fn user_gets_identity() {
	let root_key = 1u64;
	new_test_ext(root_key).execute_with(|| {
		assert_eq!(Sudo::key(), Some(root_key));
		let call = Box::new(Call::Identity(IdentityCall::create_identity {
			who: 0u64,
			name: H256::zero(),
		}));
		assert_ok!(Sudo::sudo(Origin::signed(root_key), call));
	});
}

#[test]
fn user_identity_gets_deleted() {
	let root_key = 1u64;
	new_test_ext(root_key).execute_with(|| {
		let delete_identity = Box::new(Call::Identity(IdentityCall::delete_identity { who: 0u64 }));
		let create_identity = Box::new(Call::Identity(IdentityCall::create_identity {
			who: 0u64,
			name: H256::zero(),
		}));
		assert_ok!(Sudo::sudo(Origin::signed(root_key), create_identity));
		assert_ok!(Sudo::sudo(Origin::signed(root_key), delete_identity));
	});
}

#[test]
fn check_user_has_identity() {
	let root_key = 1u64;
	new_test_ext(root_key).execute_with(|| {
		let create_identity = Box::new(Call::Identity(IdentityCall::create_identity {
			who: 0u64,
			name: H256::zero(),
		}));
		assert_ok!(Sudo::sudo(Origin::signed(root_key), create_identity));
		assert!(Identity::identities(0).is_some());
	});
}
