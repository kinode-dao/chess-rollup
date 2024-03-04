# ZK Rollup Prototype
## `elf_program`
This is the event loop of our chain that we need to prove using SP1. All boilerplate logic surrounding this code to make a "real" rollup lives in other parts of the code.

## `prover`
A kinode package that will prove a computation for you given an ELF file and some inputs. Right now it is all in wasm, and so we'll need `prover_extension` for production.

## `prover_extension`
Just a script that helps me generate proofs. It is not actually a runtime extension (yet!)

## `sequencer`
Sequencer code as a kinode app. This keeps track of state (tx history, and chain state), and uses the same code in `elf_program` to run its event loop. There is lots more boilerplate around this because it is essentially an RPC node/webserver that needs to let people send transactions to it via http
### TODO
- Bridge txs need to be real
- needs to make the data available (DAC on Kinode, posting to L1 once the new ETH provider is setup, Celestia, EigenDA, etc.)
- Frontend for demo purposes

## `sequencer_ui`
UI for the sequencer