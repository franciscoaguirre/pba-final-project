#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;
	use primitives::IdentityInterface;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<Self::AccountId>;
	}

	#[pallet::storage]
	pub type Identities<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::Hash>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		// This user is not allowed to create an identity.
		NotAuthorized,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create_identity(
			origin: OriginFor<T>,
			who: T::AccountId,
			name: T::Hash,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			ensure!(Self::check_caller(&caller), Error::<T>::NotAuthorized);
			Self::set_identity(&who, name);
			Ok(())
		}
	}
}

impl<T: Config> primitives::IdentityInterface<T::AccountId, T::Hash> for Pallet<T> {
	fn check_caller(caller: &T::AccountId) -> bool {
		true
	}

	fn set_identity(who: &T::AccountId, name: T::Hash) {
		Identities::<T>::insert(who, name);
	}

	fn get_identity(who: &T::AccountId) -> Option<T::Hash> {
		Identities::<T>::get(who)
	}
}
