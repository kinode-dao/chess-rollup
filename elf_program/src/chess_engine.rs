use crate::rollup_lib::{RollupState, TransactionData, WrappedTransaction};
use alloy_primitives::{Address as AlloyAddress, U256};
use chess::{Board, BoardStatus, ChessMove};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

type GameId = U256;

#[derive(Serialize, Deserialize)]
pub struct Game {
    turns: u64,
    board: String,
    white: AlloyAddress,
    black: AlloyAddress,
    wager: U256,
}

#[derive(Serialize, Deserialize)]
pub struct PendingGame {
    white: AlloyAddress,
    black: AlloyAddress,
    accepted: (bool, bool),
    wager: U256,
}

#[derive(Serialize, Deserialize)]
pub struct ChessState {
    pub pending_games: HashMap<GameId, PendingGame>,
    pub games: HashMap<GameId, Game>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ChessTransactions {
    ProposeGame {
        white: AlloyAddress,
        black: AlloyAddress,
        wager: U256,
    },
    StartGame(U256),
    Move {
        game_id: U256,
        san: String,
    },
    ClaimWin(U256),
}

pub fn chain_event_loop(
    tx: WrappedTransaction<ChessTransactions>,
    rollup: &mut RollupState<ChessState, ChessTransactions>,
) -> anyhow::Result<()> {
    let decode_tx = tx.clone();

    // DO NOT verify a signature for a bridge transaction
    if let TransactionData::BridgeTokens(amount) = decode_tx.data {
        rollup.balances.insert(
            tx.pub_key.clone(),
            rollup.balances.get(&tx.pub_key).unwrap_or(&U256::ZERO) + amount,
        );
        return Ok(());
    }

    if decode_tx
        .sig
        .recover_address_from_msg(&serde_json::to_string(&decode_tx.data).unwrap().as_bytes())
        .unwrap()
        != decode_tx.pub_key
    {
        return Err(anyhow::anyhow!("bad sig"));
    }

    match decode_tx.data {
        TransactionData::BridgeTokens(_) => Err(anyhow::anyhow!("shouldn't happen")),
        TransactionData::WithdrawTokens(amount) => {
            rollup.balances.insert(
                tx.pub_key.clone(),
                rollup.balances.get(&tx.pub_key).unwrap_or(&U256::ZERO) - amount,
            );
            rollup.withdrawals.push((tx.pub_key, amount));
            rollup.sequenced.push(tx);
            Ok(())
        }
        TransactionData::Transfer { from, to, amount } => {
            rollup.balances.insert(
                from.clone(),
                rollup.balances.get(&from).unwrap_or(&U256::ZERO) - amount,
            );
            rollup.balances.insert(
                to.clone(),
                rollup.balances.get(&to).unwrap_or(&U256::ZERO) + amount,
            );
            rollup.sequenced.push(tx);
            Ok(())
        }
        TransactionData::Extension(ext) => match ext {
            ChessTransactions::ProposeGame {
                white,
                black,
                wager,
            } => {
                let game_id = U256::from(rollup.state.pending_games.len());
                rollup.state.pending_games.insert(
                    game_id,
                    PendingGame {
                        white: white.clone(),
                        black: black.clone(),
                        accepted: if white == tx.pub_key {
                            (true, false)
                        } else if black == tx.pub_key {
                            (false, true)
                        } else {
                            return Err(anyhow::anyhow!("not a player"));
                        },
                        wager,
                    },
                );
                rollup.sequenced.push(tx);
                Ok(())
            }
            ChessTransactions::StartGame(game_id) => {
                let Some(pending_game) = rollup.state.pending_games.get(&game_id) else {
                    return Err(anyhow::anyhow!("game id doesn't exist"));
                };
                if pending_game.accepted == (true, false) {
                    if tx.pub_key != pending_game.black {
                        return Err(anyhow::anyhow!("not white"));
                    }
                } else if pending_game.accepted == (false, true) {
                    if tx.pub_key != pending_game.white {
                        return Err(anyhow::anyhow!("not black"));
                    }
                } else {
                    return Err(anyhow::anyhow!("impossible to reach"));
                }

                let Some(white_balance) = rollup.balances.get(&pending_game.white) else {
                    return Err(anyhow::anyhow!("white doesn't exist"));
                };
                let Some(black_balance) = rollup.balances.get(&pending_game.black) else {
                    return Err(anyhow::anyhow!("black doesn't exist"));
                };

                if white_balance < &pending_game.wager || black_balance < &pending_game.wager {
                    return Err(anyhow::anyhow!("insufficient funds"));
                }

                rollup.balances.insert(
                    pending_game.white.clone(),
                    rollup.balances.get(&pending_game.white).unwrap() - pending_game.wager,
                );
                rollup.balances.insert(
                    pending_game.black.clone(),
                    rollup.balances.get(&pending_game.black).unwrap() - pending_game.wager,
                );

                rollup.state.games.insert(
                    game_id,
                    Game {
                        turns: 0,
                        board: Board::default().to_string(),
                        white: pending_game.white.clone(),
                        black: pending_game.black.clone(),
                        wager: pending_game.wager * U256::from(2),
                    },
                );
                rollup.state.pending_games.remove(&game_id);
                rollup.sequenced.push(tx);
                Ok(())
            }
            ChessTransactions::Move { game_id, san } => {
                let Some(game) = rollup.state.games.get_mut(&game_id) else {
                    return Err(anyhow::anyhow!("game id doesn't exist"));
                };

                if game.turns % 2 == 0 && tx.pub_key != game.white {
                    return Err(anyhow::anyhow!("not white's turn"));
                } else if game.turns % 2 == 1 && tx.pub_key != game.black {
                    return Err(anyhow::anyhow!("not black's turn"));
                }

                let mut board = Board::from_str(&game.board).unwrap();
                board =
                    board.make_move_new(ChessMove::from_san(&board, &san).expect("invalid move"));
                game.board = board.to_string();
                game.turns += 1;
                rollup.sequenced.push(tx);
                Ok(())
            }
            ChessTransactions::ClaimWin(game_id) => {
                let game = rollup
                    .state
                    .games
                    .get_mut(&game_id)
                    .expect("game id doesn't exist");
                let board = Board::from_str(&game.board).unwrap();

                if board.status() == BoardStatus::Checkmate {
                    if game.turns % 2 == 0 {
                        rollup.balances.insert(
                            game.black.clone(),
                            rollup.balances.get(&game.black).unwrap() + game.wager,
                        );
                    } else {
                        rollup.balances.insert(
                            game.white.clone(),
                            rollup.balances.get(&game.white).unwrap() + game.wager,
                        );
                    }
                } else if board.status() == BoardStatus::Stalemate {
                    rollup.balances.insert(
                        game.white.clone(),
                        rollup.balances.get(&game.white).unwrap() + game.wager / U256::from(2),
                    );
                    rollup.balances.insert(
                        game.black.clone(),
                        rollup.balances.get(&game.black).unwrap() + game.wager / U256::from(2),
                    );
                } else {
                    return Err(anyhow::anyhow!("game is not over"));
                }

                rollup.state.games.remove(&game_id);
                Ok(())
            }
        },
    }
}
