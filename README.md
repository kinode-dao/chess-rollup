# ZK Rollup Prototype

## `dac`
The data availability comittee (DAC) client that hooks into the sequencer.

## `elf_program`
This is the event loop of our chain that we need to prove using SP1. All boilerplate logic surrounding this code to make a "real" rollup lives in other parts of the code.

## `prover_extension`
Runtime extension to get proofs of sp1 program

## `sequencer`
### `sequencer` process
This keeps track of state (tx history, and chain state), and uses the same code in `elf_program` to run its event loop. There is lots more boilerplate around this because it is essentially an RPC node/webserver that needs to let people send transactions to it via http. You can also run `m our@sequencer:rollup:goldfinger.kino "Prove"` to prove all the transactions using `prover_extension`.
### `disperser` process
This is responsible for "dispersing" the batch data to the data availability members. Before any batch is posted to update the L1 rollup state, the disperser sends all the state to the DAC members who sign off that they have seen it. This is then used to verify that the data was made available, reducing the changes of a withholding attack. This interacts with `client:dac:goldfinger.os` which stores and signs off on the DAC data.

## `sequencer_ui`
UI for the sequencer