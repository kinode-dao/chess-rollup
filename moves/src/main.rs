use hex::ToHex;
use ring::rand::SystemRandom;
use ring::signature::{self, KeyPair, Signature, ED25519};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct WrappedTransaction {
    pub_key: [u8; 32],
    sig: String,
    data: Vec<u8>,
    // TODO probably need to add nonces, value, gas, gasPrice, gasLimit, ... but whatever
    // I think we could use eth_sendRawTransaction to just send arbitrary bytes to a sequencer
}

#[derive(Serialize, Deserialize)]
struct BridgeTokens {
    amount: u64,
}

#[derive(Serialize, Deserialize)]
struct InitializeGame {
    white: [u8; 32],
    black: [u8; 32],
    wager: u64,
}

#[derive(Serialize, Deserialize)]
struct Move {
    game: ([u8; 32], [u8; 32]),
    san: String,
}

#[derive(Serialize, Deserialize)]
enum TxType {
    BridgeTokens(BridgeTokens),
    InitializeGame(InitializeGame),
    Move(Move),
}

fn main() {
    let seed_white = SystemRandom::new();
    let doc_white = signature::Ed25519KeyPair::generate_pkcs8(&seed_white).unwrap();
    let key_white = signature::Ed25519KeyPair::from_pkcs8(doc_white.as_ref()).unwrap();

    let seed_black = SystemRandom::new();
    let doc_black = signature::Ed25519KeyPair::generate_pkcs8(&seed_black).unwrap();
    let key_black = signature::Ed25519KeyPair::from_pkcs8(doc_black.as_ref()).unwrap();

    let bridge_white = BridgeTokens { amount: 100 };
    let bridge_white_message = bincode::serialize(&TxType::BridgeTokens(bridge_white)).unwrap();
    // let bridge_white_sig = sign_message(&bridge_white_message, &key_white);
    let bridge_white_sig: Signature = key_white.sign(&bridge_white_message);

    let bridge_black = BridgeTokens { amount: 100 };
    let bridge_black_message = bincode::serialize(&TxType::BridgeTokens(bridge_black)).unwrap();
    // let bridge_black_sig = sign_message(&bridge_black_message, &key_white);
    let bridge_black_sig: Signature = key_black.sign(&bridge_black_message);

    let white_pub_hex = key_white.public_key().as_ref().encode_hex::<String>();
    let white_bridge_message_hex = bridge_white_message.encode_hex::<String>();
    let white_signature_hex = bridge_white_sig.as_ref().encode_hex::<String>();
    println!("White Pub: {}", white_pub_hex);
    println!("White Message: {}", white_bridge_message_hex);
    println!("Signature: {}", white_signature_hex);

    let black_pub_hex = key_black.public_key().as_ref().encode_hex::<String>();
    let black_bridge_message_hex = bridge_black_message.encode_hex::<String>();
    let black_signature_hex = bridge_black_sig.as_ref().encode_hex::<String>();
    println!("Black Pub: {}", black_pub_hex);
    println!("Black Message: {}", black_bridge_message_hex);
    println!("Signature: {}", black_signature_hex);
}
