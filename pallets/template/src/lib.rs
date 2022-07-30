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
	use sp_std::vec::Vec;

	type Proposal<T> = BoundedVec<u8, <T as Config>::MaxProposalLength>;

	/// The order of the variants matter because they are stored in order in the ProposalVotes map
	#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
	pub enum Vote {
		Aye,
		Nay,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		#[pallet::constant]
		type MaxProposalLength: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn queued_proposals)]
	pub type QueuedProposals<T> = StorageValue<_, BoundedVec<Proposal<T>, ConstU32<100>>>;

	/// First is "Aye" and second is "Nay"
	#[pallet::storage]
	#[pallet::getter(fn proposal_votes)]
	pub type ProposalVotes<T> =
		StorageMap<_, Identity, <T as frame_system::Config>::Hash, (u32, u32), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn active_referendum)]
	pub type ActiveReferendum<T> = StorageValue<_, ()>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A proposal was successfully submitted
		ProposalSubmitted(Proposal<T>, T::AccountId),
		/// A vote was successfully submitted
		VoteSubmitted(Vote, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Proposal text is too long
		ProposalTooLong,
		/// Proposal queue is full
		ProposalQueueFull,
		// /// Proposal has already been submitted
		// ProposalAlreadySubmitted,
		/// No active referendum right now
		NoActiveReferendum,
		/// Overflow error
		Overflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn submit_proposal(origin: OriginFor<T>, raw_proposal: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let proposal: Proposal<T> =
				raw_proposal.try_into().map_err(|()| Error::<T>::ProposalTooLong)?;

			// TODO: Check if proposal already exists

			QueuedProposals::<T>::try_append(proposal.clone())
				.map_err(|()| Error::<T>::ProposalQueueFull)?;

			Self::deposit_event(Event::ProposalSubmitted(proposal, who));

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1))]
		pub fn submit_vote(
			origin: OriginFor<T>,
			proposal_hash: T::Hash,
			vote: Vote,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(ActiveReferendum::<T>::exists(), Error::<T>::NoActiveReferendum);

			ProposalVotes::<T>::try_mutate(proposal_hash, |(aye_votes, nay_votes)| match &vote {
				Vote::Aye => aye_votes.checked_add(1).ok_or(Error::<T>::Overflow),
				Vote::Nay => nay_votes.checked_add(1).ok_or(Error::<T>::Overflow),
			})?;

			Self::deposit_event(Event::VoteSubmitted(vote, who));

			Ok(())
		}
	}
}
