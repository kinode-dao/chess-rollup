import { useCallback } from "react";
import { ethers } from "ethers";
import { useWeb3React } from "@web3-react/core";
import { BigNumber } from 'ethers'
import useSequencerStore, { Transaction, SignedTransaction } from "../store";

interface MyGamesProps {
    baseUrl: string;
}

const MyGames = ({ baseUrl }: MyGamesProps) => {
    let { account, provider } = useWeb3React();
    const { nonces, state: { pending_games } } = useSequencerStore();

    const acceptGame = useCallback(
        async (gameId: string) => {
            try {
                if (!account || !provider) {
                    window.alert('Ethereum wallet is not connected');
                    return;
                }

                let tx: Transaction = {
                    data: {
                        Extension: {
                            StartGame: gameId,
                        }
                    },
                    nonce: nonces[account.toLowerCase()] ?
                        BigNumber.from(nonces[account.toLowerCase()]++).toHexString().replace(/^0x0+/, '0x') :
                        "0x0",
                }

                const signature = await provider.getSigner().signMessage(JSON.stringify(tx));
                const { v, r, s } = ethers.utils.splitSignature(signature);

                let wtx: SignedTransaction = {
                    pub_key: account.toLowerCase(),
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
        <div
            className="flex flex-col items-center"
        >
            <div className="flex flex-col overflow-scroll">
                {Object.keys(pending_games).map((gameId, i) => {
                    const { white, black, wager } = pending_games[gameId]; // accepted
                    if (account?.toLowerCase() == black.toLowerCase()) {
                        return (
                            <div key={i}>
                                <code>{`You have been challenged by ${white} for ${BigNumber.from(wager)} WEI`}</code>
                                <button onClick={() => acceptGame(gameId)}>Accept</button>
                            </div>
                        )
                    } else {
                        return <div key={i}></div>
                    }
                })}
            </div>
        </div>
    );
};

export default MyGames;