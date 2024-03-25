# ZK Rollup Template
## Overview
This repo contains 2 main components:
- `elf_program`: where the core logic of your rollup execution engine will live, along with its types
- `sequencer`: which contains "everything else":
  - RPC API
  - bridge interactions (deposits/withdrawals)
  - serving the web UI for your rollup
  - proving the state transition was valid (via the [prover extension](./prover_extension/))
  - (SOON) replicating data to a data availability comittee (or EigenDA/Celestia/L1, TBD)

## Developer Quick Start
(assuming your kinode is running on port 8080)
```bash
cd elf_program
cargo prove build
cd ../sequencer
kit bs
# (optional) if you want to use the prover
cd ../prover_extension
cargo run --release -- --port 8080
```
You can then go to `http://localhost:8080/sequencer:rollup:goldfinger.os` to see the UI and play around with sending transactions to your sequencer (make sure you have metamask installed in your browser!)

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