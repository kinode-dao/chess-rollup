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

## Guide to Modifying the Rollup
This chess example was written in a way to make it very easy to modify.
Most pieces can stay completely fixed with no changes.

The first thing you will need to modify is the [execution engine](./elf_program/src/chess_engine.rs).
This is where we define our state, transaction types, and insert our business logic.
There are comments peppered throughout that file to help out.

Next, you will want to modify the [sequencer_ui](./sequencer_ui/) so that it matches the app you are trying to create.
Use vite to make development easier.

That should be it, but there may be a few more things you want to modify depending on the requirements of your application.
The first is the [rpc](./sequencer/sequencer/src/rpc.rs) - which is essentially just how the frontend can interact with your rollup over HTTP (read state from the chain, post transactions to the chain, etc.).
Right now, an http `GET` returns the entire state, but you may want a more extensive/granular set of chain reads.
In addition, an http `POST` accepts a json-encoded transaction, which you also may want to switch out for something different.

Lastly, you may want to deploy your own [bridge](https://github.com/kinode-dao/chess-bridge).
Currently, the [sequencer](./sequencer/sequencer/src/lib.rs) is set up to hit a particular bridge on sepolia. This means that you will be able to read new deposits in from that rollup, but you will not be able to post withdrawals (because you do not control the sequencer key!). For development purposes, it shouldn't matter that much, but if you would like to deploy your own bridge, that option is open, and will require some modifications to the [bridge_lib](./sequencer/sequencer/src/bridge_lib.rs) once you have deployed your bridge. In the future, this will be obviated with a more automated setup to deploy your own rollup, but for now this is part of the configuration.

## Prover Extension Guide (TODO)