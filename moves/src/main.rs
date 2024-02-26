use hex::ToHex;
use ring::rand::SystemRandom;
use ring::signature::{self, KeyPair, Signature, ED25519};

fn main() {
    let seed = SystemRandom::new();
    let doc = signature::Ed25519KeyPair::generate_pkcs8(&seed).unwrap();

    let keypair = signature::Ed25519KeyPair::from_pkcs8(doc.as_ref()).unwrap();
    let message: &[u8] = b"e4";
    let sig: Signature = keypair.sign(message);

    let public_key_hex = keypair.public_key().as_ref().encode_hex::<String>();
    let message_hex = message.encode_hex::<String>();
    let signature_hex = sig.as_ref().encode_hex::<String>();

    println!("Public Key: {}", public_key_hex);
    println!("Message: {}", message_hex);
    println!("Signature: {}", signature_hex);
}
