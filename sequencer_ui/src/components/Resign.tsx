import { useCallback } from "react";
import { ethers } from "ethers";
import { useWeb3React } from "@web3-react/core";
import useSequencerStore, { Transaction, SignedTransaction } from "../store";

interface ResignProps {
    baseUrl: string;
    gameId: string;
}

const Resign = ({ baseUrl, gameId }: ResignProps) => {
    let { account, provider } = useWeb3React();
    const { nonces } = useSequencerStore();

    const resign = useCallback(
        async () => {
            try {
                if (!account || !provider) {
                    window.alert('Ethereum wallet is not connected');
                    return;
                }
                let tx: Transaction = {
                    nonce: nonces[account] ? nonces[account]++ : 0,
                    data: {
                        Extension: {
                            Resign: gameId,
                        }
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
        [account, provider]
    );


    return (
        <button onClick={resign}>
            Resign
        </button>
    );
};

export default Resign;