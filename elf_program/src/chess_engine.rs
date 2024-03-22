use crate::rollup_lib::{ExecutionEngine, RollupState, SignedTransaction, TransactionData};
use alloy_primitives::{Address as AlloyAddress, U256};
use chess::{Board, BoardStatus, ChessMove};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

type GameId = U256;

/// A game of chess
#[derive(Serialize, Deserialize)]
pub struct Game {
    turns: u64,
    board: String,
    white: AlloyAddress,
    black: AlloyAddress,
    wager: U256,
    status: String, // 0x... won, stalemate, active
}

/// A game of chess that has been proposed by white, but not accepted by black yet
#[derive(Serialize, Deserialize)]
pub struct PendingGame {
    white: AlloyAddress,
    black: AlloyAddress,
    accepted: (bool, bool),
    wager: U256,
}

/// All of the extra state we need for the chess rollup (excluding the state alread included in RollupState)
#[derive(Serialize, Deserialize)]
pub struct ChessState {
    pub pending_games: HashMap<GameId, PendingGame>,
    pub games: HashMap<GameId, Game>,
}

/// All of the transactions that will go in the TransactionData::Extension variant
/// that we need for different actions in chess
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ChessTransactions {
    ProposeGame {
        white: AlloyAddress,
        black: AlloyAddress,
        wager: U256,
    },
    StartGame(GameId),
    Move {
        game_id: GameId,
        san: String,
    },
    Resign(GameId),
}

pub type ChessRollupState = RollupState<ChessState, ChessTransactions>;

impl Default for ChessRollupState {
    fn default() -> Self {
        Self {
            sequenced: vec![],
            balances: HashMap::new(),
            withdrawals: vec![],
            nonces: HashMap::new(),
            state: ChessState {
                pending_games: HashMap::new(),
                games: HashMap::new(),
            },
        }
    }
}

impl ExecutionEngine<ChessTransactions> for ChessRollupState {
    // process a single transaction
    fn execute(&mut self, stx: SignedTransaction<ChessTransactions>) -> anyhow::Result<()> {
        let decode_stx = stx.clone();

        // DO NOT verify a signature for a bridge transaction
        if let TransactionData::BridgeTokens(amount) = decode_stx.tx.data {
            self.balances.insert(
                stx.pub_key.clone(),
                self.balances.get(&stx.pub_key).unwrap_or(&U256::ZERO) + amount,
            );
            return Ok(());
        }

        if decode_stx.tx.nonce != *self.nonces.get(&stx.pub_key).unwrap_or(&U256::ZERO) {
            return Err(anyhow::anyhow!("bad nonce"));
        } else {
            self.nonces
                .insert(stx.pub_key.clone(), decode_stx.tx.nonce + U256::from(1));
        }

        // verify the signature
        if decode_stx
            .sig
            // TODO json doesn't (de)serialize deterministically. Alternatively, use ETH RLP?
            .recover_address_from_msg(&serde_json::to_string(&decode_stx.tx).unwrap().as_bytes())
            .unwrap()
            != decode_stx.pub_key
        {
            return Err(anyhow::anyhow!("bad sig"));
        }

        // TODO check for underflows everywhere
        match decode_stx.tx.data {
            TransactionData::BridgeTokens(_) => Err(anyhow::anyhow!("shouldn't happen")),
            TransactionData::WithdrawTokens(amount) => {
                self.balances.insert(
                    stx.pub_key.clone(),
                    self.balances.get(&stx.pub_key).unwrap_or(&U256::ZERO) - amount,
                );
                self.withdrawals.push((stx.pub_key, amount));
                self.sequenced.push(stx);
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
                self.sequenced.push(stx);
                Ok(())
            }
            // TransactionData::Extension includes the business logic for the rollup
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
                            accepted: if white == stx.pub_key {
                                (true, false)
                            } else if black == stx.pub_key {
                                (false, true)
                            } else {
                                return Err(anyhow::anyhow!("not a player"));
                            },
                            wager,
                        },
                    );
                    self.sequenced.push(stx);
                    Ok(())
                }
                ChessTransactions::StartGame(game_id) => {
                    let Some(pending_game) = self.state.pending_games.get(&game_id) else {
                        return Err(anyhow::anyhow!("game id doesn't exist"));
                    };
                    if pending_game.accepted == (true, false) {
                        if stx.pub_key != pending_game.black {
                            return Err(anyhow::anyhow!("not white"));
                        }
                    } else if pending_game.accepted == (false, true) {
                        if stx.pub_key != pending_game.white {
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
                            status: "ongoing".to_string(),
                        },
                    );
                    self.state.pending_games.remove(&game_id);
                    self.sequenced.push(stx);
                    Ok(())
                }
                ChessTransactions::Move { game_id, san } => {
                    let Some(game) = self.state.games.get_mut(&game_id) else {
                        return Err(anyhow::anyhow!("game id doesn't exist"));
                    };

                    if game.turns % 2 == 0 && stx.pub_key != game.white {
                        return Err(anyhow::anyhow!("not white's turn"));
                    } else if game.turns % 2 == 1 && stx.pub_key != game.black {
                        return Err(anyhow::anyhow!("not black's turn"));
                    }

                    let mut board = Board::from_str(&game.board).unwrap();
                    let Ok(mov) = san.parse::<ChessMove>() else {
                        return Err(anyhow::anyhow!("invalid san move"));
                    };
                    board = board.make_move_new(mov);
                    game.board = board.to_string();
                    game.turns += 1;

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
                        game.status = format!("{} won", stx.pub_key);
                    } else if board.status() == BoardStatus::Stalemate {
                        self.balances.insert(
                            game.white.clone(),
                            self.balances.get(&game.white).unwrap() + game.wager / U256::from(2),
                        );
                        self.balances.insert(
                            game.black.clone(),
                            self.balances.get(&game.black).unwrap() + game.wager / U256::from(2),
                        );
                        game.status = "stalemate".to_string();
                    }

                    self.sequenced.push(stx);
                    Ok(())
                }
                ChessTransactions::Resign(game_id) => {
                    let game = self
                        .state
                        .games
                        .get_mut(&game_id)
                        .expect("game id doesn't exist");
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
                    game.status = format!("{} resigned", stx.pub_key);
                    Ok(())
                }
            },
        }
    }
}
