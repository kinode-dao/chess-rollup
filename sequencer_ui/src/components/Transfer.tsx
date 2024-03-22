import { useState, useCallback, FormEvent } from "react";
import { ethers } from "ethers";
import { useWeb3React } from "@web3-react/core";
import { BigNumber } from 'ethers'
import useSequencerStore, { Transaction, SignedTransaction } from "../store";

interface TransferProps {
    baseUrl: string;
}

const Transfer = ({ baseUrl }: TransferProps) => {
    let { account, provider } = useWeb3React();
    const { nonces } = useSequencerStore();
    const [transferTo, setTransferTo] = useState('0x6de4ff647646d9faaf1e40dcddf6ad231f696af6');
    const [transferAmount, setTransferAmount] = useState(4);

    const transfer = useCallback(
        async (e: FormEvent) => {
            e.preventDefault();
            if (!account || !provider) {
                console.log('account', account)
                console.log("provider", provider)
                console.error('Ethereum wallet is not connected');
                return;
            }

            try {
                let tx: Transaction = {
                    nonce: nonces[account] ? nonces[account]++ : 0,
                    data: {
                        Transfer: {
                            from: account.toLowerCase(),
                            to: transferTo.toLowerCase(),
                            amount: BigNumber.from(transferAmount).toHexString().replace(/^0x0+/, '0x'), // for some reason there's a leading zero...really annoying!
                        },
                    }
                }

                const signature = await provider.getSigner().signMessage(JSON.stringify(tx));
                const { v, r, s } = ethers.utils.splitSignature(signature);

                let wtx: SignedTransaction = {
                    pub_key: account,
                    sig: {
                        r, s, v
                    },
                    tx
                };

                const receipt = await fetch(`${baseUrl}/rpc`, {
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
        },
        [account, provider, transferAmount, transferTo, setTransferAmount, setTransferTo]
    );

    return (
        <div>
            <h4 className="m-2">Transfer</h4>
            <div className="flex flex-col overflow-scroll">
                <form onSubmit={transfer}>
                    <input type="text" placeholder="to" value={transferTo} onChange={(e) => setTransferTo(e.target.value)} />
                    <input
                        type="text"
                        value={transferAmount}
                        onChange={(e) => setTransferAmount(Number(e.target.value))}
                    />
                    <button type="submit">Transfer</button>
                </form>
            </div>
        </div>
    );
};

export default Transfer;