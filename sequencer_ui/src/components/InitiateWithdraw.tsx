import { useState, useCallback, FormEvent } from "react";
import { ethers } from "ethers";
import { useWeb3React } from "@web3-react/core";
import { BigNumber } from 'ethers'
import useSequencerStore, { Transaction, SignedTransaction } from "../store";

interface WithdrawProps {
    baseUrl: string;
}

const InitiateWithdraw = ({ baseUrl }: WithdrawProps) => {
    let { account, provider } = useWeb3React();
    const [amount, setAmount] = useState(0);
    const { nonces } = useSequencerStore();

    const initiateWithdraw = useCallback(
        async (e: FormEvent) => {
            e.preventDefault();
            try {
                if (!account || !provider) {
                    window.alert('Ethereum wallet is not connected');
                    return;
                }
                let tx: Transaction = {
                    data: {
                        WithdrawTokens: BigNumber.from(amount).toHexString().replace(/^0x0+/, '0x'), // for some reason there's a leading zero...really annoying!
                    },
                    nonce: nonces[account.toLowerCase()] ?
                        BigNumber.from(nonces[account.toLowerCase()]++).toHexString().replace(/^0x0+/, '0x') :
                        "0x0",
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
        [account, provider, amount]
    );


    return (
        <div>
            <h4 className="m-2">Initiate Withdraw of Tokens from Rollup</h4>
            <div className="flex flex-col overflow-scroll">
                <form onSubmit={initiateWithdraw}>
                    <input
                        type="text"
                        value={amount}
                        onChange={(e) => setAmount(Number(e.target.value))}
                    />
                    <button type="submit">Initiate Withdraw</button>
                </form>
            </div>
        </div>
    );
};

export default InitiateWithdraw;