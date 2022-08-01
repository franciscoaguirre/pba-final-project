pub trait IdentityInterface<AccountId, Name> {
	fn check_caller(caller: &AccountId) -> bool;

	fn set_identity(who: &AccountId, name: Name);

	fn get_identity(who: &AccountId) -> Option<Name>;
}
