import { useCallback } from "react";
import { ethers } from "ethers";
import { useWeb3React } from "@web3-react/core";
import { Transaction, WrappedTransaction } from "../store";

interface ResignProps {
    baseUrl: string;
    gameId: string;
}

const Resign = ({ baseUrl, gameId }: ResignProps) => {
    let { account, provider } = useWeb3React();

    const resign = useCallback(
        async () => {
            try {
                if (!account || !provider) {
                    window.alert('Ethereum wallet is not connected');
                    return;
                }
                let tx: Transaction = {
                    Extension: {
                        Resign: gameId,
                    }
                }

                const signature = await provider.getSigner().signMessage(JSON.stringify(tx));
                const { v, r, s } = ethers.utils.splitSignature(signature);

                let wtx: WrappedTransaction = {
                    pub_key: account,
                    sig: {
                        r, s, v
                    },
                    data: tx
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