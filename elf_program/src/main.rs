#![no_main]
sp1_zkvm::entrypoint!(main);

mod tx;
use tx::*;

pub fn main() {
    let mut state = sp1_zkvm::io::read::<RollupState<ChessState, ChessTransactions>>();
    let mem_pool = sp1_zkvm::io::read::<Vec<WrappedTransaction<ChessTransactions>>>();

    for tx in mem_pool.iter() {
        chain_event_loop(tx.clone(), &mut state).unwrap();
    }

    sp1_zkvm::io::write(&state);
    // NOTE/TODO we will need to write state to DA and then a hash to L1/L2
}
