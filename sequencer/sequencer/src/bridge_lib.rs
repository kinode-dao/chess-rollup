pub fn handle_log(
    state: &mut RollupState<ChessState, ChessTransactions>,
    log: &eth::Log,
) -> anyhow::Result<()> {
    // NOTE this ugliness is only because kinode_process_lib::eth is using an old version of alloy. Once it's at 0.6.3/4 we can clear this up
    match FixedBytes::<32>::new(log.topics[0].as_slice().try_into().unwrap()) {
        DepositMade::SIGNATURE_HASH => {
            println!("deposit event");
            // let event = DepositMade::from_log(&log)?;
            let deposit = DepositMade::abi_decode_data(&log.data, true).unwrap();
            let rollup_id = deposit.0;
            let token_contract = deposit.1;
            let token_id = deposit.2;
            let uqbar_dest = deposit.3;
            let amount = deposit.4;
            let _block_number = deposit.5;
            let _prev_deposit_root = deposit.6;
            if rollup_id != U256::ZERO {
                return Err(anyhow::anyhow!("not handling rollup deposits"));
            }
            if token_contract != address!("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee") {
                return Err(anyhow::anyhow!("only handling ETH deposits"));
            }
            if token_id != U256::ZERO {
                return Err(anyhow::anyhow!("not handling NFT deposits"));
            }

            chain_event_loop(
                WrappedTransaction {
                    pub_key: uqbar_dest,
                    sig: Signature::test_signature(),
                    data: TransactionData::BridgeTokens(amount),
                },
                state,
            )?;
        }
        _ => {
            return Err(anyhow::anyhow!("unknown event"));
        }
    }
    Ok(())
}
