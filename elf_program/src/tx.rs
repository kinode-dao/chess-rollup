use alloy_primitives::{Address as AlloyAddress, Signature, U256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct RollupState {
    pub sequenced: Vec<WrappedTransaction>,
    pub balances: HashMap<AlloyAddress, U256>,
    pub withdrawals: Vec<(AlloyAddress, U256)>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WrappedTransaction {
    pub pub_key: AlloyAddress,
    pub sig: Signature,
    pub data: TxType,
    // NOTE: I realized that TxType has to have a deterministic way to (de)serialize itself
    //   otherwise you'll pull your hair out wondering why the sig verification isn't working...
    //   perhaps EIP712 fixes this? Need to look. Regardless, just noting for now - fix later...
    // TODO probably need to add nonces, value, gas, gasPrice, gasLimit, ... but whatever
    // I think we could use eth_sendRawTransaction to just send arbitrary bytes to a sequencer
    // or at the very least we can use eth_signMessage plus an http request to this process
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TxType {
    BridgeTokens(U256),
    WithdrawTokens(U256),
    Transfer {
        from: AlloyAddress,
        to: AlloyAddress,
        amount: U256,
    },
    Mint {
        to: AlloyAddress,
        amount: U256,
    },
}

pub fn chain_event_loop(tx: WrappedTransaction, state: &mut RollupState) -> anyhow::Result<()> {
    let decode_tx = tx.clone();

    if decode_tx
        .sig
        .recover_address_from_msg(&serde_json::to_string(&decode_tx.data).unwrap().as_bytes())
        .unwrap()
        != decode_tx.pub_key
    {
        return Err(anyhow::anyhow!("bad sig"));
    }

    match decode_tx.data {
        TxType::BridgeTokens(_) => Ok(()),
        TxType::WithdrawTokens(amount) => {
            state.balances.insert(
                tx.pub_key.clone(),
                state.balances.get(&tx.pub_key).unwrap_or(&U256::ZERO) - amount,
            );
            state.withdrawals.push((tx.pub_key, amount));
            state.sequenced.push(tx);
            Ok(())
        }
        TxType::Transfer { from, to, amount } => {
            state.balances.insert(
                from.clone(),
                state.balances.get(&from).unwrap_or(&U256::ZERO) - amount,
            );
            state.balances.insert(
                to.clone(),
                state.balances.get(&to).unwrap_or(&U256::ZERO) + amount,
            );
            state.sequenced.push(tx);
            Ok(())
        }
        TxType::Mint { to, amount } => {
            state
                .balances
                .insert(to, state.balances.get(&to).unwrap_or(&U256::ZERO) + amount);
            state.sequenced.push(tx);
            Ok(())
        }
    }
}
