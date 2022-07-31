use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct OngoingProposalInfo<Hash, BlockNumber> {
	/// Hash of the proposal
	pub proposal_hash: Hash,
	/// When voting will end
	pub end: BlockNumber,
	/// Number of "aye" votes
	pub aye_votes: u32,
	/// Number of "nay" votes
	pub nay_votes: u32,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct FinishedProposalInfo<BlockNumber> {
	pub approved: bool,
	pub end: BlockNumber,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ProposalInfo<Hash, BlockNumber> {
	Ongoing(OngoingProposalInfo<Hash, BlockNumber>),
	Finished(FinishedProposalInfo<BlockNumber>),
}
