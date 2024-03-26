#![no_main]
sp1_zkvm::entrypoint!(main);

mod engine;
use engine::*;
mod rollup_lib;
use rollup_lib::*;

pub fn main() {
    // read in the old state
    let mut state = sp1_zkvm::io::read::<FullRollupState>();
    // read in the next batch of transactions
    let mem_pool = sp1_zkvm::io::read::<Vec<SignedTransaction<ChessTransactions>>>();

    // execute each transaction
    for tx in mem_pool.iter() {
        state.execute(tx.clone()).unwrap();
    }

    // write the new state
    sp1_zkvm::io::write(&state);

    // what happens next? For now, nothing!
    // This is only because we are currently running an authority rollup.
    // Once the verifier is out, we will take this proof and verify it on-chain to update the state
}
