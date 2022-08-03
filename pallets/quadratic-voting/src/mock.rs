use crate as pallet_quadratic_voting;
use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU32, ConstU64, GenesisBuild, Hooks},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type BlockNumber = u64;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		QuadraticVoting: pallet_quadratic_voting,
		Balances: pallet_balances,
		Identity: pallet_basic_identity,
	}
);

type Balance = u64;

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_types! {
	pub const LaunchPeriod: BlockNumber = 2;
	pub const VotingPeriod: BlockNumber = 1;
}

impl pallet_quadratic_voting::Config for Test {
	type Event = Event;
	type MaxProposalLength = ConstU32<50>;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type ProposalQueueSize = ConstU32<2>;
	type MaxVotes = ConstU32<10>;
	type Identity = Identity;
	type ProposalsPerReferendum = ConstU32<2>;
}

impl pallet_basic_identity::Config for Test {
	type Event = Event;
}

/// Builds genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_quadratic_voting::GenesisConfig::<Test> { voters: vec![1], ..Default::default() }
		.assimilate_storage(&mut t)
		.unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn next_block() {
	System::set_block_number(System::block_number() + 1);
	System::on_initialize(System::block_number());
	QuadraticVoting::on_initialize(System::block_number());
}

pub fn run_to_block(n: BlockNumber) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			QuadraticVoting::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		next_block();
	}
}
