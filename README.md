# ZK Chess
## Overview
This repo contains the prototype for a zk app-chain that lets any wallet challenge any other wallet to a game of chess.
For a quick look at the data structures we use to define this chain, what transactions look like, and how we update the state, see [tx.rs](./elf_program/src/tx.rs). This lives within the [elf program](./elf_program/) which is the compiled SP1 program we can use to generate proofs.

Of course, compiling a program to be zk provable is far from launching an app-chain, hence the [sequencer](./sequencer) kinode package.
This kinode package handles "everything else" - handling new transactions, letting clients read the state of the chain via http, handling bridging, serving [a UI](./sequencer_ui/), etc.
Under the hood, the core event-loop is the exact same logic as what is in [tx.rs](./elf_program/src/tx.rs), which lets us prove later on that we executed this appchain properly.

When we finally want that proof, we hook the sequencer up to a [prover extension](./prover_extension/) which will take the list of transactions and the event-loop program, and produce a proof that the sequencer calculated the new state correctly.

## Components

- `elf_program`: This is the event loop of our chain that we need to prove using SP1.

- `prover_extension`: Runtime extension to get proofs of sp1 program

- `sequencer`: 
This keeps track of state (tx history, and chain state), and uses the same code in `elf_program` to run its event loop.
  - `m our@sequencer:rollup:goldfinger.kino "Prove"` to prove all the transactions using (make sure you also have the `prover_extension` setup!)

- `sequencer_ui`: UI for the sequencer
- `dac`: work in progress for data availability using kinode multisigs

## Setup
First, build the `elf_program`, which we use in the sequencer to zk prove the chain:
```bash
cd elf_program
cargo prove build
```
Next, make sure you have a kinode running (use `kit f` or run a local node), build and install the sequencer
```bash
cd ../sequencer
kit bs
```
Next, if you navigate to `<YOUR_NODE_URL>/sequencer:rollup:goldfinger.os`, you can start sending transactions to the local chain!

If you want to prove your transactions, build and run the `prover_extension`
```bash
cd ../prover_extension
cargo run --release -- --port 8080
```
You should see
```bash
rollup:goldfinger.os: sequencer: connected to prover_extension
```
in your kinode's terminal.

Finally, to prove your transaction history, simply run
```bash
m our@sequencer:rollup:goldfinger.kino "Prove"
```
in your kinode's terminal, and the proof.json file will be in `<HOME_DIRECTORY>/vfs/rollup:goldfinger.os/proofs/proof.json`.

## Work in Progress Components
### `dac`
The data availability comittee (DAC) client that hooks into the sequencer.
Other nodes can assist the core sequencer in providing data availability.
The idea is that instead of posting all of our data to L1 blobs, which is expensive, we can delegate that to a multisig to get much more data throughput.
### `disperser` process
This is responsible for "dispersing" the batch data to the data availability members.
Before any batch is posted to update the L1 rollup state, the disperser sends all the state to the DAC members who sign off that they have seen it.
This is then used to verify that the data was made available, reducing the changes of a withholding attack.
This interacts with `client:dac:goldfinger.os` which stores and signs off on the DAC data.
