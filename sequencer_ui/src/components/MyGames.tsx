import { useCallback } from "react";
import { ethers } from "ethers";
import { useWeb3React } from "@web3-react/core";
import { BigNumber } from 'ethers'
import useSequencerStore, { Transaction, SignedTransaction } from "../store";
import { Chessboard } from "react-chessboard";
import { Chess } from "chess.js";
import Resign from "./Resign";

interface MyGamesProps {
    baseUrl: string;
}

const MyGames = ({ baseUrl }: MyGamesProps) => {
    let { account, provider } = useWeb3React();
    const { nonces, state, set } = useSequencerStore();

    const onDrop = useCallback(
        (sourceSquare: string, targetSquare: string, gameId: string) => {
            if (!account || !provider) {
                window.alert('Ethereum wallet is not connected');
                return false;
            }
            if (!gameId || !state.games[gameId]) return false;

            try {
                let { board, white, black } = state.games[gameId];
                const chess = new Chess(board);

                if (chess.isGameOver()) return false;
                if (chess.turn() == 'w' && account.toLowerCase() != white.toLowerCase()) return false;
                if (chess.turn() == 'b' && account.toLowerCase() != black.toLowerCase()) return false;

                const result = chess.move({
                    from: sourceSquare,
                    to: targetSquare,
                    promotion: 'q'
                });

                if (result == null) {
                    return false;
                }
                board = chess.fen();
                state.games[gameId].board = board;
                set({ state });

                let tx: Transaction = {
                    data: {
                        Extension: {
                            Move: {
                                game_id: gameId,
                                san: `${sourceSquare}${targetSquare}`,
                            },
                        }
                    },
                    nonce: nonces[account.toLowerCase()] ?
                        BigNumber.from(nonces[account.toLowerCase()]++).toHexString().replace(/^0x0+/, '0x') :
                        "0x0",
                }

                provider.getSigner().signMessage(JSON.stringify(tx)).then((signature) => {
                    const { v, r, s } = ethers.utils.splitSignature(signature);

                    let wtx: SignedTransaction = {
                        pub_key: account!,
                        sig: {
                            r, s, v
                        },
                        tx
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
                    }).catch(err => {
                        console.error(err);
                        alert("There was an error making your move. Please try again");
                        // TODO need to undo the FEN in the state
                        return false;
                    });
                    return false;
                });
                return false
            } catch (err) {
                console.error(err);
                return false
            }
        },
        [account, provider, state, set]
    );

    return (
        <div
            className="flex flex-col items-center"
        >
            <h4 className="m-2">Active Games</h4>
            <div className="flex flex-col overflow-auto">
                {Object.keys(state.games).map((gameId, i) => {
                    const { status, turns, board, white, black, wager } = state.games[gameId]; // accepted
                    const whoMovesNext = turns % 2 == 0 ? white.toLowerCase() : black.toLowerCase();
                    const boardOrientation = account?.toLowerCase() == black.toLowerCase() ? 'black' : 'white';
                    if (status != 'ongoing') {
                        return (
                            <code key={i}>{`Game ${gameId} is over, ${status}`}</code>
                        )
                    } else if (account?.toLowerCase() == white.toLowerCase() || account?.toLowerCase() == black.toLowerCase()) {
                        return (
                            <div key={i}>
                                <p>{
                                    account == whoMovesNext ?
                                        `Your move vs ${account == white.toLowerCase() ? black : white}` :
                                        `Waiting for ${account == white.toLowerCase() ? black : white} to move`
                                }</p>
                                <Chessboard
                                    // boardWidth={boardWidth - 16}
                                    position={board}
                                    onPieceDrop={(source, target, _) => onDrop(source, target, gameId)}
                                    boardOrientation={boardOrientation}
                                />
                                <Resign baseUrl={baseUrl} gameId={gameId} />
                            </div>
                        )
                    } else {
                        return <code key={i}>{`${gameId}: ${white} vs ${black} for ${BigNumber.from(wager)} WEI`}</code>
                    }
                })}
            </div>
        </div>
    );
};

export default MyGames;