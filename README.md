# ZK Rollup Template
## Overview
This repo contains 3 components:
- `elf_program`: where the core logic of your rollup execution engine will live, along with its types
- `prover_extension`: which you can run alongside the `sequencer` to prove the state of your rollup (this repo DOES NOT need to be modified in any way)
- `sequencer`: which contains "everything else":
  - RPC API definition and handling
  - sending messages to the `prover_extension`
  - subscribing/handling deposit events on L1
  - serving the web UI for your rollup
  - (SOON) replicating data to a data availability layer

## Quick Start
(assuming your kinode is running on port 8080)
```bash
cd elf_program
cargo prove build
cd ../sequencer
kit bs
cd ../prover_extension
cargo run --release -- --port 8080
```
You can then go to `http://localhost:5173/sequencer:rollup:goldfinger.os` to see the UI and play around with sending transactions to your sequencer (make sure you have metamask installed in your browser!)

## Developer Guide
### `elf_program`
Start in the `elf_program`.
[main.rs](./elf_program/src/main.rs) and [rollup_lib.rs](./elf_program/src/rollup_lib.rs) can stay the same, do not edit them.
They will be libraries that you import one day, but for now just leave them alone.
In [chess_engine.rs](./elf_program/src/chess_engine.rs), you can see how we build around the `rollup_lib` by defining our own custom `ChessState` and `ChessTransactions`, and then implementing the `ExecutionEngine` trait for `RollupState<ChessState, ChessTransactions>` to wrap everything together.
The `execute` function is the core logic of our chain, which you can modify at your leisure.

### `sequencer`
Editing the `elf_program` for your needs is essentially the "toy" version of the app.
All of the productionization of that chain now has to live in the Kinode sequencer package. 
The code is reasonably well documented, I would reccomend just reading the comments to get a feel for it