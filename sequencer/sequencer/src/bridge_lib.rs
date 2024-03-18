use crate::{ChessRollupState, ExecutionEngine, RollupState, TransactionData, WrappedTransaction};
use alloy_primitives::{address, Signature, U256};
use alloy_sol_types::{sol, SolEvent};
use kinode_process_lib::eth;
use kinode_process_lib::println;

sol! {
    event DepositMade(uint256 indexed town, address indexed tokenContract, address indexed uqbarDest,
        uint256 tokenId, uint256 amount, uint256 blockNumber, bytes32 prevDepositRoot
    );
}

/// TODO this needs to include a town_id parameter so that you can filter by *just*
/// the deposits to the rollup you care about
pub fn subscribe_to_logs(eth_provider: &eth::Provider) {
    let filter = eth::Filter::new()
        .address(
            "0xC61655415f3d8d8eE527b45aEc7101A9c6083A81"
                .parse::<eth::Address>()
                .unwrap(),
        )
        .from_block(5436837)
        .to_block(eth::BlockNumberOrTag::Latest)
        .events(vec![
            "DepositMade(uint256,address,address,uint256,uint256,uint256,bytes32)",
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

/// TODO this needs to include a town_id
/// TODO this needs to include a from_block parameter because we don't want to reprocess
pub fn get_old_logs(eth_provider: &eth::Provider, state: &mut ChessRollupState) {
    let filter = eth::Filter::new()
        .address(
            "0xC61655415f3d8d8eE527b45aEc7101A9c6083A81"
                .parse::<eth::Address>()
                .unwrap(),
        )
        .from_block(5436837)
        .to_block(eth::BlockNumberOrTag::Latest)
        .events(vec![
            "DepositMade(uint256,address,address,uint256,uint256,uint256,bytes32)",
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
        DepositMade::SIGNATURE_HASH => {
            println!("deposit event");
            let rollup_id: U256 = log.topics[1].into();
            let token_contract = eth::Address::from_word(log.topics[2]);
            let uqbar_dest = eth::Address::from_word(log.topics[3]);
            // let event = DepositMade::from_log(&log)?;
            let deposit = DepositMade::abi_decode_data(&log.data, true).unwrap();
            let token_id = deposit.0;
            let amount = deposit.1;
            let _block_number = deposit.2;
            let _prev_deposit_root = deposit.3;
            if rollup_id != U256::ZERO {
                return Err(anyhow::anyhow!("not handling rollup deposits"));
            }
            if token_contract != address!("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee") {
                return Err(anyhow::anyhow!("only handling ETH deposits"));
            }
            if token_id != U256::ZERO {
                return Err(anyhow::anyhow!("not handling NFT deposits"));
            }

            state.execute(WrappedTransaction {
                pub_key: uqbar_dest,
                sig: Signature::test_signature(),
                data: TransactionData::BridgeTokens(amount),
            })?;
        }
        _ => {
            return Err(anyhow::anyhow!("unknown event"));
        }
    }
    Ok(())
}
