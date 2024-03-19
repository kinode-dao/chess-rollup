import { create } from 'zustand'
import { persist, createJSONStorage } from 'zustand/middleware'

export interface PendingGame {
  white: string;
  black: string;
  accepted: [boolean, boolean];
  wager: string;
}

export interface Game {
  turns: number;
  board: string;
  white: string;
  black: string;
  wager: string;
  // could add move history?
}

export interface WrappedTransaction {
  pub_key: string; // Converted camelCase for TypeScript conventions
  sig: Sig;
  data: Transaction; // Still a hex string, but consider using ArrayBuffer or similar for binary data handling in JS/TS
  // Additional fields like nonces, value, gas, gasPrice, gasLimit, etc., can be added as needed.
}

export type Sig = {
  r: string;
  s: string;
  v: number;
};

// For the `Transaction` enum, TypeScript uses a combination of types and interfaces to achieve similar functionality.
export type Transaction =
  | {
    Transfer: {
      from: string;
      to: string;
      amount: string; // BigNumber
    }
  }
  | {
    Extension: | {
      ProposeGame: {
        white: string;
        black: string;
        wager: string; // BigNumber
      }
    }
    | {
      StartGame: string;
    }
    | {
      Move: {
        game_id: string;
        san: string;
      }
    }
    | {
      ClaimWin: string;
    }
    | {
      Resign: string;
    }
  }

export interface SequencerStore {
  sequenced: WrappedTransaction[]
  balances: Record<string, number>
  withdrawals: any, // TODO
  state: {
    pending_games: Record<string, PendingGame>
    games: Record<string, Game>
  }
  set: (partial: SequencerStore | Partial<SequencerStore>) => void
}

const useSequencerStore = create<SequencerStore>()(
  persist(
    (set) => ({  // get
      sequenced: [],
      balances: {},
      withdrawals: [],
      state: {
        pending_games: {},
        games: {},
      },
      set,
    }),
    {
      name: 'sequencer', // unique name
      storage: createJSONStorage(() => localStorage), // (optional) by default, 'localStorage' is used
    }
  )
)

export default useSequencerStore
