import { ethers } from 'ethers';
import { Transaction, SignedTransaction } from './store';

export default async function sendTx(tx: Transaction, account: string, rpcUrl: string) {
    try {
        if (!window.ethereum) {
            console.error('Ethereum wallet is not connected');
            return;
        }

        const signature = await window.ethereum.request({
            method: 'personal_sign',
            params: [JSON.stringify(tx), account],
        });
        const { v, r, s } = ethers.utils.splitSignature(signature);

        let wtx: SignedTransaction = {
            pub_key: account,
            sig: {
                r, s, v
            },
            tx
        };

        const receipt = await fetch(rpcUrl, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(wtx),
        });
        console.log('receipt', receipt);
    } catch (err) {
        console.error(err);
    }
}