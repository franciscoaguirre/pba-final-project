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
	use sp_core::Hasher;
	use sp_runtime::traits::{Saturating, Zero};
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

		#[pallet::constant]
		type ProposalQueueSize: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn queued_proposals)]
	pub type QueuedProposals<T: Config> =
		StorageValue<_, BoundedVec<Proposal<T>, T::ProposalQueueSize>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn referendum_count)]
	pub type ReferendumCount<T> = StorageValue<_, ReferendumIndex, ValueQuery>;

	/// Storage info of all finished and ongoing referenda.
	/// Inside each referenda, multiple proposals could be being voted on.
	/// Twox64Concat is fine to use because referendum_index and proposal_index
	/// are not controlled by a user.
	#[pallet::storage]
	#[pallet::getter(fn referendum_info)]
	pub type ReferendumInfo<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		ReferendumIndex,
		Twox64Concat,
		ProposalIndex,
		ProposalInfo<T::Hash, T::BlockNumber>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn referendum_ends_at)]
	pub type ReferendumEndsAt<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

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
		/// Started a referendum
		ReferendumStarted(ReferendumIndex),
		/// Referendum ended
		ReferendumEnded(ReferendumIndex),
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
		/// Tried to start a referendum but there were no proposals in the queue
		NoProposalsInQueue,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(block_number: BlockNumberFor<T>) -> Weight {
			let referendum_ends_at = ReferendumEndsAt::<T>::get();

			if block_number == referendum_ends_at {
				let _ = Self::end_referendum(); // TODO: Deal with error
			}

			if (block_number % T::LaunchPeriod::get()).is_zero() {
				let _ = Self::start_referendum(block_number); // TODO: Deal with error
			}

			// TODO: What weight should I return here?

			0
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn submit_proposal(origin: OriginFor<T>, raw_proposal: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let proposal: Proposal<T> =
				raw_proposal.try_into().map_err(|()| Error::<T>::ProposalTooLong)?;

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

			let maybe_info = ReferendumInfo::<T>::get(current_referendum, proposal_index);
			let mut proposal_info =
				maybe_info.expect("We already checked current referendum exists; qed");
			match proposal_info {
				ProposalInfo::Finished(_) => panic!(
					"We already checked current referendum
			exists; qed"
				),
				ProposalInfo::Ongoing(ref mut ongoing_info) => match &vote {
					Vote::Aye => {
						let aye_votes = ongoing_info
							.tally
							.aye_votes
							.checked_add(1)
							.ok_or(Error::<T>::Overflow)?;
						ongoing_info.tally.aye_votes = aye_votes;
					},
					Vote::Nay => {
						let nay_votes = ongoing_info
							.tally
							.nay_votes
							.checked_add(1)
							.ok_or(Error::<T>::Overflow)?;
						ongoing_info.tally.nay_votes = nay_votes;
					},
				},
			};
			ReferendumInfo::<T>::insert(current_referendum, proposal_index, proposal_info);

			Self::deposit_event(Event::VoteSubmitted(vote, who));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn start_referendum(block_number: T::BlockNumber) -> DispatchResult {
			// Update referendum index
			let referendum_index = Self::referendum_count();

			// Build proposal info struct
			// TODO: Handle multiple proposals per referendum
			let proposal_index = 0;
			let mut queued_proposals = Self::queued_proposals();

			ensure!(queued_proposals.len() > 0, Error::<T>::NoProposalsInQueue);

			let proposal_hash = <<T as frame_system::Config>::Hashing as Hasher>::hash(
				&queued_proposals.remove(proposal_index),
			);

			ReferendumEndsAt::<T>::put(block_number.saturating_add(T::VotingPeriod::get()));

			let ongoing_proposal_info =
				OngoingProposalInfo { proposal_hash, tally: Default::default() };
			let proposal_info = ProposalInfo::Ongoing(ongoing_proposal_info);

			// Insert new proposal info
			ReferendumInfo::<T>::insert(
				referendum_index,
				proposal_index as ProposalIndex,
				proposal_info,
			);

			ActiveReferendum::<T>::put(());

			Self::deposit_event(Event::<T>::ReferendumStarted(referendum_index));

			Ok(())
		}

		fn end_referendum() -> DispatchResult {
			let referendum_index = Self::referendum_count();
			let end = Self::referendum_ends_at();

			// TODO: Handle multiple proposals in the future
			let proposal_index = 0 as ProposalIndex;

			let old_proposal_info = ReferendumInfo::<T>::get(referendum_index, proposal_index)
				.expect("referendum is ending, old proposal exists; qed");

			let approved = match old_proposal_info {
				ProposalInfo::Ongoing(ongoing_proposal_info) =>
					ongoing_proposal_info.tally.result(),
				ProposalInfo::Finished(_) => panic!("Old proposal has to be ongoing; qed"),
			};

			let new_proposal_info = ProposalInfo::Finished(FinishedProposalInfo { approved, end });

			ReferendumInfo::<T>::insert(referendum_index, proposal_index, new_proposal_info);

			ActiveReferendum::<T>::kill();
			ReferendumCount::<T>::put(referendum_index + 1);

			Self::deposit_event(Event::<T>::ReferendumEnded(referendum_index));

			Ok(())
		}
	}
}
