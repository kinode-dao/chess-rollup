import { useState, useCallback, FormEvent } from "react";
import { ethers } from "ethers";
import { useWeb3React } from "@web3-react/core";
import { BigNumber } from 'ethers'
import useSequencerStore, { Transaction, SignedTransaction } from "../store";

interface ProposeGameProps {
    baseUrl: string;
}

const ProposeGame = ({ baseUrl }: ProposeGameProps) => {
    let { account, provider } = useWeb3React();
    const { nonces } = useSequencerStore();
    const [black, setBlack] = useState('0x6de4ff647646d9faaf1e40dcddf6ad231f696af6');
    const [wager, setWager] = useState(4);

    const proposeGame = useCallback(
        async (e: FormEvent) => {
            e.preventDefault();
            try {
                if (!account || !provider) {
                    window.alert('Ethereum wallet is not connected');
                    return;
                }
                let tx: Transaction = {
                    data: {
                        Extension: {
                            ProposeGame: {
                                white: account.toLowerCase(),
                                black: black.toLowerCase(),
                                wager: BigNumber.from(wager).toHexString().replace(/^0x0+/, '0x'), // for some reason there's a leading zero...really annoying!
                            },
                        }
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
        [account, provider, black, wager]
    );


    return (
        <div>
            <h4 className="m-2">Propose Game</h4>
            <div className="flex flex-col overflow-auto">
                <form onSubmit={proposeGame} className="flex flex-col">
                    <div className="flex">
                        <input
                            type="text"
                            placeholder="opponent"
                            value={black}
                            onChange={(e) => setBlack(e.target.value)}
                            className="w-3/4"
                        />
                        <input
                            type="text"
                            value={wager}
                            onChange={(e) => setWager(Number(e.target.value))}
                            className="w-1/4"
                        />
                    </div>
                    <button type="submit" className="mt-2">Propose Game</button>
                </form>
            </div>
        </div>
    );
};

export default ProposeGame;