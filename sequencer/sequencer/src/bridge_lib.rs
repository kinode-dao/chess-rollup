use crate::{ChessRollupState, ExecutionEngine, RollupState, TransactionData, WrappedTransaction};
use alloy_primitives::{address, Signature, U256};
use alloy_sol_types::{sol, SolEvent};
use kinode_process_lib::eth;
use kinode_process_lib::println;

sol! {
    event Deposit(address sender, uint256 amount);
}

/// TODO this needs to include a town_id parameter so that you can filter by *just*
/// the deposits to the rollup you care about
pub fn subscribe_to_logs(eth_provider: &eth::Provider, rollup_id: U256) {
    let filter = eth::Filter::new()
        .address(
            "0xA25489Af7c695DE69eDd19F7A688B2195B363f23"
                .parse::<eth::Address>()
                .unwrap(),
        )
        .topic1(rollup_id)
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
pub fn get_old_logs(eth_provider: &eth::Provider, state: &mut ChessRollupState, rollup_id: U256) {
    let filter = eth::Filter::new()
        .address(
            "0xA25489Af7c695DE69eDd19F7A688B2195B363f23"
                .parse::<eth::Address>()
                .unwrap(),
        )
        .topic1(rollup_id)
        .from_block(5436837)
        .to_block(eth::BlockNumberOrTag::Latest)
        .events(vec!["Deposit(address,uint256)"]);
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
            let rollup_id: U256 = log.topics[1].into();
            // let token_contract = eth::Address::from_word(log.topics[2]);
            // let uqbar_dest = eth::Address::from_word(log.topics[3]);
            // let event = DepositMade::from_log(&log)?;
            let deposit = Deposit::abi_decode_data(&log.data, true).unwrap();
            let sender = deposit.0;
            let amount = deposit.1;

            state.execute(WrappedTransaction {
                pub_key: sender,
                sig: Signature::test_signature(), // TODO should be a zero sig...
                data: TransactionData::BridgeTokens(amount),
            })?;
        }
        _ => {
            return Err(anyhow::anyhow!("unknown event"));
        }
    }
    Ok(())
}
