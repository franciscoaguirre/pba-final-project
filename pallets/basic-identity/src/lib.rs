#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use primitives::IdentityInterface;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::storage]
	#[pallet::getter(fn identities)]
	pub type Identities<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::Hash>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// User has been given an identity
		IdentityCreated(T::AccountId, T::Hash),
		/// Identity has been removed
		IdentityDeleted(T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// No identity
		IdentityDoesNotExist,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_identity(
			origin: OriginFor<T>,
			who: T::AccountId,
			name: T::Hash,
		) -> DispatchResult {
			ensure_root(origin)?;
			Self::set_identity(&who, name);
			Self::deposit_event(Event::<T>::IdentityCreated(who, name));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn delete_identity(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(Self::has_identity(&who), Error::<T>::IdentityDoesNotExist);
			Self::clear_identity(&who);
			Self::deposit_event(Event::<T>::IdentityDeleted(who));
			Ok(())
		}
	}
}

impl<T: Config> primitives::IdentityInterface<T::AccountId, T::Hash> for Pallet<T> {
	fn has_identity(who: &T::AccountId) -> bool {
		Identities::<T>::get(who).is_some()
	}

	fn set_identity(who: &T::AccountId, name: T::Hash) {
		Identities::<T>::insert(who, name);
	}

	fn clear_identity(who: &T::AccountId) {
		Identities::<T>::remove(who);
	}

	fn get_identity(who: &T::AccountId) -> Option<T::Hash> {
		Identities::<T>::get(who)
	}
}
