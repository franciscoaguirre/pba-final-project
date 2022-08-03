# Polkadot Blockchain Academy - Final Project

## Quadratic Voting

This project allows users to submit proposals and vote on many of them at the same time,
deciding how to distribute their voting power (points) in a [quadratic manner](https://www.economist.com/interactive/2021/12/18/quadratic-voting).

## Pallets

- pallet-quadratic-voting: This implements the quadratic voting logic.
  Users need to register as voters before they begin submitting anything.
  To do this they need an identity from the basic identity pallet.
  Voters can submit proposals in plain text, these are put in a queue.
  Every N blocks (configurable, by default 10), a referendum will start
  and pick M proposals (also configurable, default 10) from the queue for voters to vote on.
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
