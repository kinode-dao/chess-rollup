use alloy_primitives::{Address as AlloyAddress, Signature};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct RollupState {
    pub sequenced: Vec<WrappedTransaction>,
    pub balances: HashMap<String, u64>,
    pub withdrawals: Vec<(String, u64)>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WrappedTransaction {
    // all these are hex strings, maybe move to alloy types at some point
    pub pub_key: String,
    pub sig: String,
    pub data: TxType,
    // TODO probably need to add nonces, value, gas, gasPrice, gasLimit, ... but whatever
    // I think we could use eth_sendRawTransaction to just send arbitrary bytes to a sequencer
    // or at the very least we can use eth_signMessage plus an http request to this process
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TxType {
    BridgeTokens(u64),   // TODO U256
    WithdrawTokens(u64), // TODO U256
    Transfer {
        from: String,
        to: String,
        amount: u64, // TODO U256
    },
    Mint {
        to: String,
        amount: u64, // TODO U256
    },
}

pub fn chain_event_loop(tx: WrappedTransaction, state: &mut RollupState) -> anyhow::Result<()> {
    let decode_tx = tx.clone();
    let pub_key_bytes: [u8; 20] = hex::decode(&decode_tx.pub_key).unwrap().try_into().unwrap();
    // let pub_key: AlloyAddress = AlloyAddress::from(pub_key_bytes);
    // let signature: Signature = bincode::deserialize(&hex::decode(&decode_tx.sig).unwrap()).unwrap();

    // if signature.recover_address_from_msg(&data_bytes[..]).unwrap() != pub_key_bytes {
    //     return Err(anyhow::anyhow!("bad sig"));
    // }

    match decode_tx.data {
        TxType::BridgeTokens(amount) => {
            state.balances.insert(
                tx.pub_key.clone(),
                state.balances.get(&tx.pub_key).unwrap_or(&0) + amount,
            );
            state.sequenced.push(tx);
            Ok(())
        }
        TxType::WithdrawTokens(amount) => {
            state.balances.insert(
                tx.pub_key.clone(),
                state.balances.get(&tx.pub_key).unwrap_or(&0) - amount,
            );
            state.withdrawals.push((tx.pub_key.clone(), amount));
            state.sequenced.push(tx);
            Ok(())
        }
        TxType::Transfer { from, to, amount } => {
            state.balances.insert(
                from.clone(),
                state.balances.get(&from).unwrap_or(&0) - amount,
            );
            state
                .balances
                .insert(to.clone(), state.balances.get(&to).unwrap_or(&0) + amount);
            state.sequenced.push(tx);
            Ok(())
        }
        TxType::Mint { to, amount } => {
            state
                .balances
                .insert(to.clone(), state.balances.get(&to).unwrap_or(&0) + amount);
            state.sequenced.push(tx);
            Ok(())
        }
    }
}
