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

/// TODO this needs to include a town_id parameter so that you can filter by *just*
/// the deposits to the rollup you care about
pub fn subscribe_to_logs(eth_provider: &eth::Provider) {
    let filter = eth::Filter::new()
        .address(
            "0xA25489Af7c695DE69eDd19F7A688B2195B363f23"
                .parse::<eth::Address>()
                .unwrap(),
        )
        .from_block(5436837)
        .to_block(eth::BlockNumberOrTag::Latest)
        .events(vec!["Deposit(address,uint256)"]);

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

/// TODO this needs to include a town_id
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
            // let rollup_id: U256 = log.topics[1].into();
            // let token_contract = eth::Address::from_word(log.topics[2]);
            // let uqbar_dest = eth::Address::from_word(log.topics[3]);
            // let event = DepositMade::from_log(&log)?;
            let deposit = Deposit::abi_decode_data(&log.data, true).unwrap();
            let sender = deposit.0;
            let amount = deposit.1;

            state.execute(SignedTransaction {
                pub_key: sender,
                sig: Signature::test_signature(), // TODO should be a zero sig...
                tx: Transaction {
                    nonce: U256::ZERO, // NOTE: this doesn't need to be a "real" nonce since bridge txs are ex-nihilo
                    data: TransactionData::BridgeTokens(amount),
                },
            })?;
        }
        BatchPosted::SIGNATURE_HASH => {
            println!("batch event");
            let batch = BatchPosted::abi_decode_data(&log.data, true).unwrap();
            let index: usize = batch.0.to::<usize>();
            let root = batch.1;

            if state.batches[index].root == root {
                println!("batch processed");
                state.batches[index].verified = true;
                return Ok(());
            } else {
                println!("TODO put this batch into some waiting period, load in state later");
            }
        }
        _ => {
            return Err(anyhow::anyhow!("unknown event"));
        }
    }
    Ok(())
}
