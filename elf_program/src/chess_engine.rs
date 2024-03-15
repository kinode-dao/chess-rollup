use crate::rollup_lib::{ExecutionEngine, RollupState, TransactionData, WrappedTransaction};
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

impl ExecutionEngine<ChessTransactions> for RollupState<ChessState, ChessTransactions> {
    fn execute(&mut self, tx: WrappedTransaction<ChessTransactions>) -> anyhow::Result<()> {
        let decode_tx = tx.clone();

        // DO NOT verify a signature for a bridge transaction
        if let TransactionData::BridgeTokens(amount) = decode_tx.data {
            self.balances.insert(
                tx.pub_key.clone(),
                self.balances.get(&tx.pub_key).unwrap_or(&U256::ZERO) + amount,
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
                self.balances.insert(
                    tx.pub_key.clone(),
                    self.balances.get(&tx.pub_key).unwrap_or(&U256::ZERO) - amount,
                );
                self.withdrawals.push((tx.pub_key, amount));
                self.sequenced.push(tx);
                Ok(())
            }
            TransactionData::Transfer { from, to, amount } => {
                self.balances.insert(
                    from.clone(),
                    self.balances.get(&from).unwrap_or(&U256::ZERO) - amount,
                );
                self.balances.insert(
                    to.clone(),
                    self.balances.get(&to).unwrap_or(&U256::ZERO) + amount,
                );
                self.sequenced.push(tx);
                Ok(())
            }
            TransactionData::Extension(ext) => match ext {
                ChessTransactions::ProposeGame {
                    white,
                    black,
                    wager,
                } => {
                    let game_id = U256::from(self.state.pending_games.len());
                    self.state.pending_games.insert(
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
                    self.sequenced.push(tx);
                    Ok(())
                }
                ChessTransactions::StartGame(game_id) => {
                    let Some(pending_game) = self.state.pending_games.get(&game_id) else {
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

                    let Some(white_balance) = self.balances.get(&pending_game.white) else {
                        return Err(anyhow::anyhow!("white doesn't exist"));
                    };
                    let Some(black_balance) = self.balances.get(&pending_game.black) else {
                        return Err(anyhow::anyhow!("black doesn't exist"));
                    };

                    if white_balance < &pending_game.wager || black_balance < &pending_game.wager {
                        return Err(anyhow::anyhow!("insufficient funds"));
                    }

                    self.balances.insert(
                        pending_game.white.clone(),
                        self.balances.get(&pending_game.white).unwrap() - pending_game.wager,
                    );
                    self.balances.insert(
                        pending_game.black.clone(),
                        self.balances.get(&pending_game.black).unwrap() - pending_game.wager,
                    );

                    self.state.games.insert(
                        game_id,
                        Game {
                            turns: 0,
                            board: Board::default().to_string(),
                            white: pending_game.white.clone(),
                            black: pending_game.black.clone(),
                            wager: pending_game.wager * U256::from(2),
                        },
                    );
                    self.state.pending_games.remove(&game_id);
                    self.sequenced.push(tx);
                    Ok(())
                }
                ChessTransactions::Move { game_id, san } => {
                    let Some(game) = self.state.games.get_mut(&game_id) else {
                        return Err(anyhow::anyhow!("game id doesn't exist"));
                    };

                    if game.turns % 2 == 0 && tx.pub_key != game.white {
                        return Err(anyhow::anyhow!("not white's turn"));
                    } else if game.turns % 2 == 1 && tx.pub_key != game.black {
                        return Err(anyhow::anyhow!("not black's turn"));
                    }

                    let mut board = Board::from_str(&game.board).unwrap();
                    board = board
                        .make_move_new(ChessMove::from_san(&board, &san).expect("invalid move"));
                    game.board = board.to_string();
                    game.turns += 1;
                    self.sequenced.push(tx);
                    Ok(())
                }
                ChessTransactions::ClaimWin(game_id) => {
                    let game = self
                        .state
                        .games
                        .get_mut(&game_id)
                        .expect("game id doesn't exist");
                    let board = Board::from_str(&game.board).unwrap();

                    if board.status() == BoardStatus::Checkmate {
                        if game.turns % 2 == 0 {
                            self.balances.insert(
                                game.black.clone(),
                                self.balances.get(&game.black).unwrap() + game.wager,
                            );
                        } else {
                            self.balances.insert(
                                game.white.clone(),
                                self.balances.get(&game.white).unwrap() + game.wager,
                            );
                        }
                    } else if board.status() == BoardStatus::Stalemate {
                        self.balances.insert(
                            game.white.clone(),
                            self.balances.get(&game.white).unwrap() + game.wager / U256::from(2),
                        );
                        self.balances.insert(
                            game.black.clone(),
                            self.balances.get(&game.black).unwrap() + game.wager / U256::from(2),
                        );
                    } else {
                        return Err(anyhow::anyhow!("game is not over"));
                    }

                    self.state.games.remove(&game_id);
                    Ok(())
                }
            },
        }
    }
}
