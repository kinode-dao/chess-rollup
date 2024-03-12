import { ethers } from 'ethers';
import { TxType, WrappedTransaction } from './store';

export default async function sendTx(tx: TxType, account: string, rpcUrl: string) {
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

        let wtx: WrappedTransaction = {
            pub_key: account,
            sig: {
                r, s, v
            },
            data: tx
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