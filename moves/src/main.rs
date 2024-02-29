use secp256k1::hashes::sha256;
use secp256k1::rand::rngs::OsRng;
use secp256k1::{Message, Secp256k1};
use serde::{Deserialize, Serialize};

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

fn main() {
    let secp = Secp256k1::new();

    let (white_secret, white_public) = secp.generate_keypair(&mut OsRng);
    let (black_secret, black_public) = secp.generate_keypair(&mut OsRng);

    let bridge_white = BridgeTokens { amount: 100 };
    let bridge_white_message = Message::from_hashed_data::<sha256::Hash>(
        &bincode::serialize(&TxType::BridgeTokens(bridge_white)).unwrap(),
    );
    let bridge_white_sig = secp.sign_ecdsa(&bridge_white_message, &white_secret);

    let bridge_black = BridgeTokens { amount: 100 };
    let bridge_black_message = Message::from_hashed_data::<sha256::Hash>(
        &bincode::serialize(&TxType::BridgeTokens(bridge_black)).unwrap(),
    );
    let bridge_black_sig = secp.sign_ecdsa(&bridge_black_message, &black_secret);

    println!("White Pub:     {}", hex::encode(white_public.serialize()));
    println!("White Message: {}", bridge_white_message);
    println!(
        "Signature:     {}",
        hex::encode(bridge_white_sig.serialize_compact())
    );

    println!("Black Pub:     {}", hex::encode(black_public.serialize()));
    println!("Black Message: {}", bridge_black_message);
    println!(
        "Signature:     {}",
        hex::encode(bridge_black_sig.serialize_compact())
    );
}
