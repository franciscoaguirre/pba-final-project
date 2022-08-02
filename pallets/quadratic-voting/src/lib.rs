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

use frame_support::{dispatch::Weight, pallet_prelude::*, traits::ReservableCurrency};
use frame_system::pallet_prelude::{BlockNumberFor, *};
use primitives::IdentityInterface;
use sp_core::Hasher;
use sp_runtime::traits::{Saturating, Zero};
use sp_std::vec::Vec;

pub type ReferendumIndex = u32;
pub type ProposalIndex = u8;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	type Proposal<T> = BoundedVec<u8, <T as Config>::MaxProposalLength>;
	type Points = u32;

	/// Vote possibilities
	#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
	pub enum Vote {
		Aye,
		Nay,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Proposals can't be more than this length
		#[pallet::constant]
		type MaxProposalLength: Get<u32>;

		/// Size of the proposal queue
		#[pallet::constant]
		type ProposalQueueSize: Get<u32>;

		/// How often (in blocks) new referenda are ran
		#[pallet::constant]
		type LaunchPeriod: Get<Self::BlockNumber>;

		/// How long (in blocks) referenda allow votes for until they end
		#[pallet::constant]
		type VotingPeriod: Get<Self::BlockNumber>;

		/// Number of proposals to be voted per referendum
		#[pallet::constant]
		type ProposalsPerReferendum: Get<u32>;

		/// Maximum votes a voter can use on any proposal in a referendum
		#[pallet::constant]
		type MaxVotes: Get<u32>;

		/// AccountId used for testing, will be a part of the voter group
		#[pallet::constant]
		type TestVoter: Get<Self::AccountId>;

		/// Currency for making proposal deposits
		/// TODO: Not yet implemented
		type Currency: ReservableCurrency<Self::AccountId>;

		/// Identity pallet, used to allow users to register as voters
		type Identity: IdentityInterface<Self::AccountId, Self::Hash>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Proposals that are queued to be used in the next referendum
	/// T::ProposalsPerReferendum have to be queued for a referendum to start
	#[pallet::storage]
	#[pallet::getter(fn queued_proposals)]
	pub type QueuedProposals<T: Config> =
		StorageValue<_, BoundedVec<Proposal<T>, T::ProposalQueueSize>, ValueQuery>;

	/// Referenda that have taken place thus far, also works as an index to the last referendum
	#[pallet::storage]
	#[pallet::getter(fn referendum_count)]
	pub type ReferendumCount<T> = StorageValue<_, ReferendumIndex, ValueQuery>;

	/// Storage info of all finished and ongoing referenda.
	/// Inside each referendum, T::ProposalsPerReferendum are voted on.
	/// Twox64Concat is fine to use here because referendum_index and proposal_index
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

	/// Time (in blocks) when the current referendum (if any) will end.
	/// Currently, only one referendum can be active at a time.
	/// TODO: Allow more than one?
	#[pallet::storage]
	#[pallet::getter(fn referendum_ends_at)]
	pub type ReferendumEndsAt<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	/// Is there a referendum active right now?
	/// TODO: Could derive this from other storage items.
	#[pallet::storage]
	#[pallet::getter(fn active_referendum)]
	pub type ActiveReferendum<T: Config> = StorageValue<_, ()>;

	/// Defines the set of all votes.
	/// Users need to call `register_voter` to end up here.
	/// Each voter has T::MaxVotes ** 2 points and only has less when he voted on a referendum.
	/// Points are returned once the referendum ends.
	#[pallet::storage]
	#[pallet::getter(fn voter_points)]
	pub type VoterPoints<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Points>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		_phantom: sp_std::marker::PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { _phantom: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			T::Identity::set_identity(&T::TestVoter::get(), T::Hash::default());
			Pallet::<T>::do_register_voter(T::TestVoter::get())
				.expect("test voter identity set in genesis; qed");
		}
	}

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
		/// No active referendum right now
		NoActiveReferendum,
		/// Overflow error
		Overflow,
		/// Tried to start a referendum but there were no proposals in the queue
		NoProposalsInQueue,
		/// User is not part of the voter group
		NotAVoter,
		/// Tried to vote more than T::MaxVotes
		TooManyVotes,
		/// Not enough points for the votes requested
		NotEnoughPoints,
		/// Voter has already been registered
		VoterAlreadyRegistered,
		/// Already voted
		AlreadyVoted,
		/// User does not have an identity
		NoIdentity,
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

			ensure!(Self::is_a_voter(&who), Error::<T>::NotAVoter);

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
			number_of_votes: u32,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Self::is_a_voter(&who), Error::<T>::NotAVoter);
			ensure!(ActiveReferendum::<T>::exists(), Error::<T>::NoActiveReferendum);
			ensure!(!Self::already_voted(&who), Error::<T>::AlreadyVoted);
			ensure!(Self::has_enough_points(&who, number_of_votes), Error::<T>::NotEnoughPoints);

			let current_referendum = ReferendumCount::<T>::get();

			let maybe_info = ReferendumInfo::<T>::get(current_referendum, proposal_index);
			let mut proposal_info =
				maybe_info.expect("We already checked current referendum exists; qed");
			match proposal_info {
				ProposalInfo::Finished(_) =>
					panic!("We already checked current referendum exists; qed"),
				ProposalInfo::Ongoing(ref mut ongoing_info) => match &vote {
					Vote::Aye => {
						let aye_votes = ongoing_info
							.tally
							.aye_votes
							.checked_add(number_of_votes)
							.ok_or(Error::<T>::Overflow)?;
						ongoing_info.tally.aye_votes = aye_votes;
					},
					Vote::Nay => {
						let nay_votes = ongoing_info
							.tally
							.nay_votes
							.checked_add(number_of_votes)
							.ok_or(Error::<T>::Overflow)?;
						ongoing_info.tally.nay_votes = nay_votes;
					},
				},
			};
			Self::take_points_from_voter(&who, number_of_votes);
			ReferendumInfo::<T>::insert(current_referendum, proposal_index, proposal_info);

			Self::deposit_event(Event::VoteSubmitted(vote, who));

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn register_voter(_: OriginFor<T>, account: T::AccountId) -> DispatchResult {
			Self::do_register_voter(account)
		}
	}
}

impl<T: Config> Pallet<T> {
	fn do_register_voter(account: T::AccountId) -> DispatchResult {
		ensure!(T::Identity::has_identity(&account), Error::<T>::NoIdentity);
		ensure!(VoterPoints::<T>::get(&account) == None, Error::<T>::VoterAlreadyRegistered);
		VoterPoints::<T>::insert(account, 100);
		Ok(())
	}

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
			ProposalInfo::Ongoing(ongoing_proposal_info) => ongoing_proposal_info.tally.result(),
			ProposalInfo::Finished(_) => panic!("Old proposal has to be ongoing; qed"),
		};

		let new_proposal_info = ProposalInfo::Finished(FinishedProposalInfo { approved, end });

		ReferendumInfo::<T>::insert(referendum_index, proposal_index, new_proposal_info);

		ActiveReferendum::<T>::kill();
		ReferendumCount::<T>::put(referendum_index + 1);

		Self::deposit_event(Event::<T>::ReferendumEnded(referendum_index));

		Ok(())
	}

	fn is_a_voter(account: &T::AccountId) -> bool {
		VoterPoints::<T>::get(account).is_some()
	}

	fn has_enough_points(account: &T::AccountId, intended_votes: u32) -> bool {
		let points_available =
			VoterPoints::<T>::get(account).expect("check should be done outside this function");
		points_available >= intended_votes.pow(2)
	}

	fn take_points_from_voter(account: &T::AccountId, votes: u32) {
		let available_points =
			VoterPoints::<T>::get(account).expect("check should be done outside this function");
		// TODO: Use checked_sub? `has_enough_points` should've been called already
		VoterPoints::<T>::insert(account, available_points - votes.pow(2));
	}

	// TODO: Let them change their vote
	// For that I need to store what they voted before
	fn already_voted(account: &T::AccountId) -> bool {
		let available_points =
			VoterPoints::<T>::get(account).expect("check should be done outside this function");
		available_points < T::MaxVotes::get().pow(2)
	}
}
