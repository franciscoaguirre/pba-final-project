use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Tally {
	/// Number of "aye" votes
	pub aye_votes: u32,
	/// Number of "nay" votes
	pub nay_votes: u32,
}

impl Tally {
    pub fn result(&self) -> bool {
        self.aye_votes > self.nay_votes
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct OngoingProposalInfo<Hash> {
	/// Hash of the proposal
	pub proposal_hash: Hash,
	/// Tally of the votes
	pub tally: Tally,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct FinishedProposalInfo<BlockNumber> {
	pub approved: bool,
	pub end: BlockNumber,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ProposalInfo<Hash, BlockNumber> {
	Ongoing(OngoingProposalInfo<Hash>),
	Finished(FinishedProposalInfo<BlockNumber>),
}
