import { useCallback } from "react";
import { ethers } from "ethers";
import { useWeb3React } from "@web3-react/core";
import { BigNumber } from 'ethers'
import useSequencerStore, { Game, TxType, WrappedTransaction } from "../store";
import { Chessboard } from "react-chessboard";

interface MyGamesProps {
    baseUrl: string;
}

const MyGames = ({ baseUrl }: MyGamesProps) => {
    let { account, provider } = useWeb3React();
    const { games } = useSequencerStore();

    const onDrop = useCallback(
        (sourceSquare: string, targetSquare: string, piece: any, gameId: string) => {
            try {
                if (!account || !provider) {
                    window.alert('Ethereum wallet is not connected');
                    return false;
                }
                console.log('san', `${sourceSquare}${targetSquare}`)
                let tx: TxType = {
                    Move: {
                        game_id: gameId,
                        san: `${sourceSquare}${targetSquare}`,
                    },
                }

                provider.getSigner().signMessage(JSON.stringify(tx)).then((signature) => {
                    const { v, r, s } = ethers.utils.splitSignature(signature);

                    let wtx: WrappedTransaction = {
                        pub_key: account!,
                        sig: {
                            r, s, v
                        },
                        data: tx
                    };

                    fetch(`${baseUrl}/rpc`, {
                        method: "POST",
                        headers: {
                            "Content-Type": "application/json",
                        },
                        body: JSON.stringify(wtx),
                    }).then((receipt) => {
                        console.log('receipt', receipt);
                        return true;
                    }).catch(console.error);
                    return false;
                });
                return false
            } catch (err) {
                console.error(err);
                return false
            }
        },
        [account, provider, games]
    );

    return (
        <div
            className="flex flex-col items-center"
        >
            <h4 className="m-2">Active Games</h4>
            <div className="flex flex-col overflow-scroll">
                {Object.keys(games).map((gameId, i) => {
                    const { turns, board, white, black, wager } = games[gameId]; // accepted
                    if (account?.toLowerCase() == white.toLowerCase() ||
                        account?.toLowerCase() == black.toLowerCase()) {
                        if (turns % 2 == 0 && account.toLowerCase() == white.toLowerCase()) {
                            return (
                                <div key={i}>
                                    <p>{`Your move vs ${black}`}</p>
                                    <Chessboard
                                        // boardWidth={boardWidth - 16}
                                        position={board}
                                        onPieceDrop={(source, target, piece) => onDrop(source, target, piece, gameId)}
                                        boardOrientation="white"
                                    />
                                </div>
                            )
                        } else if (turns % 2 == 1 && account.toLowerCase() == black.toLowerCase()) {
                            return (
                                <div key={i}>
                                    <p>{`Your move vs ${white}`}</p>
                                    <Chessboard
                                        // boardWidth={boardWidth - 16}
                                        position={board}
                                        onPieceDrop={(source, target, piece) => onDrop(source, target, piece, gameId)}
                                        boardOrientation="black"
                                    />
                                </div>
                            )
                        } else {
                            return (
                                <div key={i}>
                                    <code>{`Waiting for ${turns % 2 == 0 ? white : black} to move`}</code>
                                    <Chessboard
                                        // boardWidth={boardWidth - 16}
                                        position={board}
                                        onPieceDrop={(_) => false}
                                        boardOrientation={turns % 2 == 0 ? 'black' : 'white'}
                                    />
                                </div>
                            )
                        }
                    } else {
                        return <code key={i}>{`${gameId}: ${white} vs ${black} for ${BigNumber.from(wager)} WEI`}</code>
                    }
                })}
            </div>
        </div>
    );
};

export default MyGames;