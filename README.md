# ZK Rollup Template
## Overview
This repo contains 4 main components:
- `elf_program`: where the core logic of your rollup execution engine will live, along with its types
- `sequencer`: which contains "everything else":
  - RPC API
  - bridge interactions (deposits/withdrawals)
  - serving the web UI for your rollup
  - proving the state transition was valid (via the [prover extension](./prover_extension/))
  - (SOON) replicating data to a data availability comittee (or EigenDA/Celestia/L1, TBD)
- `sequencer_ui`: the UI for the sequencer app (in this case a chess app)
- `prover_extension`: an optional runtime extension for zk-proving the state transition of the app (this will become more relevant once the EVM verifier is released and connected to the bridge)

## Developer Quick Start
(assuming your kinode is running on port 8080)
```bash
cd ./sequencer
kit bs
# (optional) if you want to use the prover
cd ../prover_extension
cargo run --release -- --port 8080
```
You can then go to `http://localhost:8080/sequencer:rollup:goldfinger.os` to see the UI and play around with sending transactions to your sequencer (make sure you have metamask installed in your browser!)

## Guide to Modifying the Rollup
This chess example was written in a way to make it very easy to modify.
Most pieces can stay completely fixed with no changes.

The first thing you will need to modify is the [execution engine](./elf_program/src/engine.rs).
This is where we define our state, transaction types, and insert our business logic.
There are comments peppered throughout that file to help out.
Your `FullRollupState` must implement the `ExecutionEngine` trait with the following methods:
- `load`: loading the state from kinode (most projects can leave this as is)
- `save`: saving the state to kinode (most projects can leave this as is)
- `rpc`: for handling chain reads/writes over http (you may want to modify this slightly, but it is fine to leave as is)
- `execute`: the most important part, which will execute a single transaction
After editing `ExecutionEngine` `impl` to fit your new application, you can simply build and install the app on your kinode with `kit bs`.

Next, you will want to modify the [sequencer_ui](./sequencer_ui/) so that it matches the app you are trying to create.
Use vite to make development easier.

Lastly, you may want to deploy your own [bridge](https://github.com/kinode-dao/chess-bridge).
Currently, the [sequencer](./sequencer/sequencer/src/lib.rs) is set up to hit a particular bridge on Optimism.
This means that you will be able to read new deposits in from that rollup, but you will not be able to post withdrawals (because you do not control the sequencer key!).
For development purposes, it shouldn't matter that much, but if you would like to deploy your own bridge, that option is open, and will require some modifications to the [bridge_lib](./sequencer/sequencer/src/bridge_lib.rs) once you have deployed your bridge.
In the future, this will be obviated with a more automated setup to deploy your own rollup, but for now this is part of the configuration.

## Prover Extension Guide (TODO)