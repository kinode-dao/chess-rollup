use crate::{
    ChessRollupState, ExecutionEngine, RollupState, SignedTransaction, Transaction, TransactionData,
};
use alloy_primitives::{Signature, U256};
use alloy_sol_types::{sol, SolEvent};
use kinode_process_lib::eth;
use kinode_process_lib::println;

sol! {
    event Deposit(address sender, uint256 amount);
    event BatchPosted(uint256 withdrawRootIndex, bytes32 withdrawRoot);
}

pub fn subscribe_to_logs(eth_provider: &eth::Provider) {
    let filter = eth::Filter::new()
        .address(
            "0xA25489Af7c695DE69eDd19F7A688B2195B363f23"
                .parse::<eth::Address>()
                .unwrap(),
        )
        .from_block(5436837)
        .to_block(eth::BlockNumberOrTag::Latest)
        .events(vec![
            "Deposit(address,uint256)",
            "BatchPosted(uint256,bytes32)",
        ]);

    loop {
        match eth_provider.subscribe(1, filter.clone()) {
            Ok(()) => break,
            Err(_) => {
                println!("failed to subscribe to chain! trying again in 5s...");
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }
        }
    }
    println!("subscribed to logs successfully");
}

/// TODO this needs to include a from_block parameter because we don't want to reprocess
pub fn get_old_logs(eth_provider: &eth::Provider, state: &mut ChessRollupState) {
    let filter = eth::Filter::new()
        .address(
            "0xA25489Af7c695DE69eDd19F7A688B2195B363f23"
                .parse::<eth::Address>()
                .unwrap(),
        )
        .from_block(5436837)
        .to_block(eth::BlockNumberOrTag::Latest)
        .events(vec![
            "Deposit(address,uint256)",
            "BatchPosted(uint256,bytes32)",
        ]);
    loop {
        match eth_provider.get_logs(&filter) {
            Ok(logs) => {
                for log in logs {
                    match handle_log(state, &log) {
                        Ok(()) => continue,
                        Err(e) => println!("error handling log: {:?}", e),
                    }
                }
                break;
            }
            Err(_) => {
                println!("failed to fetch logs! trying again in 5s...");
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }
        }
    }
}

pub fn handle_log<S, T>(state: &mut RollupState<S, T>, log: &eth::Log) -> anyhow::Result<()>
where
    RollupState<S, T>: ExecutionEngine<T>,
{
    match log.topics[0] {
        Deposit::SIGNATURE_HASH => {
            println!("deposit event");
            let deposit = Deposit::abi_decode_data(&log.data, true).unwrap();
            let sender = deposit.0;
            let amount = deposit.1;

            state.execute(SignedTransaction {
                pub_key: sender,
                sig: Signature::test_signature(), // NOTE: deposit txs are unsigned (TODO should be a null sig)
                tx: Transaction {
                    nonce: U256::ZERO, // NOTE: this doesn't need to be a "real" nonce since deposits are ex-nihilo
                    data: TransactionData::BridgeTokens(amount),
                },
            })?;
        }
        BatchPosted::SIGNATURE_HASH => {
            let batch = BatchPosted::abi_decode_data(&log.data, true).unwrap();
            let index: usize = batch.0.to::<usize>();
            let root = batch.1;

            if state.batches[index].root == root {
                state.batches[index].verified = true;
                return Ok(());
            } else {
                // If this ever happens, it means the sequencer is in an inconsistent state with the chain
                println!("sequencer: critical error, state out of sync with chain");
            }
        }
        _ => {
            return Err(anyhow::anyhow!("unknown event"));
        }
    }
    Ok(())
}
