#![no_main]
sp1_zkvm::entrypoint!(main);

use chess::{Board, ChessMove};
use ed25519_dalek::*;
use serde::{Deserialize, Serialize};
use std::hint::black_box;
use std::str::FromStr;

#[derive(Serialize, Deserialize)]
struct Move {
    san: String,
    sig: String,
}

pub fn main() {
    let white: [u8; 32] = hex::decode(sp1_zkvm::io::read::<String>())
        .unwrap()
        .try_into()
        .unwrap();
    let black: [u8; 32] = hex::decode(sp1_zkvm::io::read::<String>())
        .unwrap()
        .try_into()
        .unwrap();
    let moves = sp1_zkvm::io::read::<Vec<Move>>();

    let mut b = Board::default();

    let mut turn = true;
    for mov in moves.iter() {
        let msg_bytes = hex::decode(&mov.san).unwrap();
        let sig_bytes = hex::decode(&mov.sig).unwrap();

        let verifying_key = if turn {
            black_box(VerifyingKey::from_bytes(&white).unwrap())
        } else {
            black_box(VerifyingKey::from_bytes(&black).unwrap())
        };
        let sig1 = black_box(Signature::try_from(&sig_bytes[..]).unwrap());
        assert!(verifying_key
            .verify_strict(&black_box(msg_bytes), &black_box(sig1))
            .is_ok());

        let san_move = ChessMove::from_san(&b, &mov.san);
        if !san_move.is_ok() {
            panic!("Invalid move");
        };
        b = b.make_move_new(san_move.unwrap());
        turn = !turn;
    }

    // Write winner (true/false for white/black ig)
    // TODO calculate score?
    sp1_zkvm::io::write(&true);
}
