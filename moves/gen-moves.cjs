const nacl = require('tweetnacl');
nacl.util = require('tweetnacl-util');

// Function to generate a signer, sign a message, and print the results
function generateAndSignMessage(keyPair, message) {
    const uint8ArrayMessage = nacl.util.decodeUTF8(message);
    const signature = nacl.sign(uint8ArrayMessage, keyPair.secretKey);

    // Convert the signature and public key to base64 for easier display
    const priv = Buffer.from(keyPair.secretKey).toString('hex');
    const sig = Buffer.from(signature).toString('hex');
    const pub = Buffer.from(keyPair.publicKey).toString('hex');

    console.log(`Signer: ${priv}`)
    console.log(`Public Key: ${pub}`);
    console.log(`Message: ${message}, ${Buffer.from(message).toString('hex')}`);
    console.log(`Signature: ${sig}\n`);
}
const white = nacl.sign.keyPair();
const black = nacl.sign.keyPair();

console.log("Signer 1:");
generateAndSignMessage(white, 'e4');

console.log("Signer 2:");
generateAndSignMessage(black, 'c5');
