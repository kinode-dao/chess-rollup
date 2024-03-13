import { useCallback } from "react";
import { ethers } from "ethers";
import { useWeb3React } from "@web3-react/core";
import { BigNumber } from 'ethers'
import useSequencerStore, { TxType, WrappedTransaction } from "../store";

interface MyGamesProps {
    baseUrl: string;
}

const MyGames = ({ baseUrl }: MyGamesProps) => {
    let { account, provider } = useWeb3React();
    const { pending_games } = useSequencerStore();

    const acceptGame = useCallback(
        async (gameId: string) => {
            let tx: TxType = {
                StartGame: gameId,
            }
            try {
                if (!account || !provider) {
                    window.alert('Ethereum wallet is not connected');
                    return;
                }

                const signature = await provider.getSigner().signMessage(JSON.stringify(tx));
                const { v, r, s } = ethers.utils.splitSignature(signature);

                let wtx: WrappedTransaction = {
                    pub_key: account.toLowerCase(),
                    sig: {
                        r, s, v
                    },
                    data: tx
                };

                const receipt = await fetch(baseUrl, {
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
                                <p>{`You have been challenged by ${white} for ${BigNumber.from(wager)} WEI`}</p>
                                <button onClick={() => acceptGame(gameId)}>Accept</button>
                            </div>
                        )
                    } else {
                        return <></>
                    }
                })}
            </div>
        </div>
    );
};

export default MyGames;