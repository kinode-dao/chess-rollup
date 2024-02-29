#![no_main]
sp1_zkvm::entrypoint!(main);

use chess::{Board, ChessMove};
// use ed25519_dalek::*;
use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hint::black_box;
use std::str::FromStr;

#[derive(Serialize, Deserialize)]
struct WrappedTransaction {
    // all these are hex strings, maybe move to alloy types at some point
    pub_key: String,
    sig: String,
    data: String, // hex string
                  // TODO probably need to add nonces, value, gas, gasPrice, gasLimit, ... but whatever
                  // I think we could use eth_sendRawTransaction to just send arbitrary bytes to a sequencer
}

#[derive(Serialize, Deserialize)]
struct BridgeTokens {
    amount: u64,
}

#[derive(Serialize, Deserialize)]
struct InitializeGame {
    white: String,
    black: String,
    wager: u64,
}

#[derive(Serialize, Deserialize)]
struct Move {
    game: (String, String),
    san: String,
}

#[derive(Serialize, Deserialize)]
enum TxType {
    BridgeTokens(BridgeTokens),
    InitializeGame(InitializeGame),
    Move(Move),
}

pub fn main() {
    let mut balances = sp1_zkvm::io::read::<HashMap<String, u64>>();
    let mut games = sp1_zkvm::io::read::<HashMap<(String, String), (String, u64)>>(); // just a map of FEN encoded boards, wager

    let mem_pool = sp1_zkvm::io::read::<Vec<WrappedTransaction>>();

    for tx in mem_pool.iter() {
        let pub_key_bytes = hex::decode(&tx.pub_key).unwrap();
        let sig_bytes = hex::decode(&tx.sig).unwrap();
        let data_bytes = hex::decode(&tx.data).unwrap();

        // let pub_key = black_box(VerifyingKey::from_bytes(&tx.pub_key).unwrap());

        // let pub_key = VerifyingKey::from_sec1_bytes(&tx.pub_key).unwrap();
        // if !pub_key
        //     .verify(&data_bytes, &Signature::try_from(&sig_bytes[..]).unwrap())
        //     .is_ok()
        // {
        //     sp1_zkvm::io::write(&"bad sig");
        //     panic!("bad sig");
        // }

        let Ok(tx_bytes) = hex::decode(tx.data.as_str()) else {
            sp1_zkvm::io::write(&"bad tx bytes");
            panic!("bad tx bytes");
        };
        let Ok(tx_type) = bincode::deserialize(&tx_bytes) else {
            sp1_zkvm::io::write(&"bad tx type");
            panic!("bad tx type");
        };
        // TODO verify tx sigs
        match tx_type {
            TxType::BridgeTokens(bridge_tokens) => {
                balances.insert(
                    tx.pub_key.clone(),
                    balances.get(&tx.pub_key).unwrap_or(&0) + bridge_tokens.amount,
                );
                sp1_zkvm::io::write(&format!("bridged {}", bridge_tokens.amount));
            }
            TxType::InitializeGame(initialize_game) => {
                assert!(balances.get(&initialize_game.white).unwrap() >= &initialize_game.wager);
                assert!(balances.get(&initialize_game.black).unwrap() >= &initialize_game.wager);
                // TODO probably need to lock tokens somewhere
                games.insert(
                    (initialize_game.white, initialize_game.black),
                    (Board::default().to_string(), initialize_game.wager),
                );
                sp1_zkvm::io::write(&"initialize game");
            }
            TxType::Move(mov) => {
                sp1_zkvm::io::write(&"move");
                let (board_raw, wager) = games.get(&mov.game).unwrap();
                let board = Board::from_str(board_raw.as_str()).unwrap();
                let san_move = ChessMove::from_san(&board, mov.san.as_str());
                if !san_move.is_ok() {
                    sp1_zkvm::io::write(&"bad move");
                    panic!("bad move");
                };
                let board = board.make_move_new(san_move.unwrap());
                games.insert((mov.game.0, mov.game.1), (board.to_string(), *wager));
                sp1_zkvm::io::write(&"move made");
                // TODO if checkmate, redistribute funds
            }
        }
    }
}
