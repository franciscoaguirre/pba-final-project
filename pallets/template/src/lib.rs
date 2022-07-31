#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod types;

pub use types::{FinishedProposalInfo, OngoingProposalInfo, ProposalInfo};

pub type ReferendumIndex = u32;
pub type ProposalIndex = u8;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::Weight, pallet_prelude::*};
	use frame_system::pallet_prelude::{BlockNumberFor, *};
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

		#[pallet::constant]
		type LaunchPeriod: Get<Self::BlockNumber>;

		#[pallet::constant]
		type VotingPeriod: Get<Self::BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn queued_proposals)]
	pub type QueuedProposals<T: Config> = StorageValue<_, BoundedVec<Proposal<T>, ConstU32<100>>>;

	#[pallet::storage]
	#[pallet::getter(fn referendum_count)]
	pub type ReferendumCount<T> = StorageValue<_, ReferendumIndex, ValueQuery>;

	/// Storage info of all finished and ongoing referenda.
	/// Inside each referenda, multiple proposals could be being voted on.
	/// Twox64Concat is fine to use because referendum_index and proposal_index
	/// are not controlled by a user.
	#[pallet::storage]
	#[pallet::getter(fn proposal_votes)]
	pub type ReferendumInfo<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		ReferendumIndex,
		Twox64Concat,
		ProposalIndex,
		ProposalInfo<T::Hash, T::BlockNumber>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn active_referendum)]
	pub type ActiveReferendum<T: Config> = StorageValue<_, ()>;

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
		/// Vote submitted too early
		TooEarly,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(block_number: BlockNumberFor<T>) -> Weight {
			let max_block_weight = T::BlockWeights::get().max_block;

			if block_number % T::LaunchPeriod::get() == 0u32.into() {
				// Start a referendum
				ActiveReferendum::<T>::put(());
			} else if block_number % T::VotingPeriod::get() == 0u32.into() {
				ActiveReferendum::<T>::kill();
			}

			// TODO: What weight should I return here?

			max_block_weight
		}
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

		/// Submit vote for a certain proposal in the current referendum
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1))]
		pub fn submit_vote(
			origin: OriginFor<T>,
			proposal_index: ProposalIndex,
			vote: Vote,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(ActiveReferendum::<T>::exists(), Error::<T>::NoActiveReferendum);

			let current_referendum = ReferendumCount::<T>::get();

			ReferendumInfo::<T>::try_mutate(current_referendum, proposal_index, |maybe_info| {
				if let Some(proposal_info) = maybe_info {
					match proposal_info {
						ProposalInfo::Finished(_) =>
							panic!("We already checked current referendum exists; qed"),
						ProposalInfo::Ongoing(ongoing_info) => match &vote {
							Vote::Aye =>
								ongoing_info.aye_votes.checked_add(1).ok_or(Error::<T>::Overflow),
							Vote::Nay =>
								ongoing_info.nay_votes.checked_add(1).ok_or(Error::<T>::Overflow),
						},
					}
				} else {
					Err(Error::<T>::TooEarly)
				}
			})?;

			Self::deposit_event(Event::VoteSubmitted(vote, who));

			Ok(())
		}
	}
}
