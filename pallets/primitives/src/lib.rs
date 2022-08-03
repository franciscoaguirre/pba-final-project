#![cfg_attr(not(feature = "std"), no_std)]

/// Identity interface for loosely coupling the basic identity pallet
pub trait IdentityInterface<AccountId, Name> {
	fn has_identity(who: &AccountId) -> bool;

	fn set_identity(who: &AccountId, name: Name);

	fn get_identity(who: &AccountId) -> Option<Name>;

	fn clear_identity(who: &AccountId);
}
