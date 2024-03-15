#![no_main]
sp1_zkvm::entrypoint!(main);

mod chess_engine;
use chess_engine::*;
mod rollup_lib;
use rollup_lib::*;

pub fn main() {
    let mut state = sp1_zkvm::io::read::<RollupState<ChessState, ChessTransactions>>();
    let mem_pool = sp1_zkvm::io::read::<Vec<WrappedTransaction<ChessTransactions>>>();

    for tx in mem_pool.iter() {
        state.execute(tx.clone()).unwrap();
    }

    sp1_zkvm::io::write(&state);
    // NOTE/TODO we will need to write state to DA and then a hash to L1/L2
}
