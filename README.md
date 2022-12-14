# Polkadot Blockchain Academy - Final Project

## Quadratic Voting

This project allows users to submit proposals and vote on many of them at the same time,
deciding how to distribute their voting power (points) in a [quadratic manner](https://www.economist.com/interactive/2021/12/18/quadratic-voting).

## Pallets

- pallet-quadratic-voting: This implements the quadratic voting logic.
  Users need to register as voters before they begin submitting anything.
  To do this they need an identity from the basic identity pallet.
  Voters can submit proposals in plain text, these are put in a queue.
  Every N blocks (configurable, by default 3), a referendum will start
  and pick M proposals (also configurable, default 2) from the queue for voters to vote on.
  When there's an active referendum running, voters can submit votes for each of the proposals
  on that referendum.

- pallet-basic-identity: Basic identity pallet that uses a root account to create and delete
  identities.

## Getting Started

### Run

To run the node, run the following command.

```sh
cargo run --release -- --dev
```

### Tests

```sh
cargo test -p pallet-quadratic-voting
```

```sh
cargo test -p pallet-basic-identity
```

## Things to improve

- Make a frontend! (need to get better with polkadot.js)
- Store hash of proposals on-chain to not have to compute it each time
- Store a map from hashes to proposal text to allow frontend to see the text
- Have voters put down a deposit when submitting a proposal so as to not spam the network
- Allow multiple referenda to be held at the same time
- Optimize `on_initialize` as much as possible and remove possible panics (expect, looking at you)
  Do less work, not allow a referendum to end and a new one to start on the same block
- Benchmarking
