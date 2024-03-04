import { create } from 'zustand'
import { persist, createJSONStorage } from 'zustand/middleware'

export interface WrappedTransaction {
  pub_key: string; // Converted camelCase for TypeScript conventions
  sig: Sig;
  data: TxType; // Still a hex string, but consider using ArrayBuffer or similar for binary data handling in JS/TS
  // Additional fields like nonces, value, gas, gasPrice, gasLimit, etc., can be added as needed.
}

export type Sig = {
  r: string;
  s: string;
  v: number;
};

// For the `TxType` enum, TypeScript uses a combination of types and interfaces to achieve similar functionality.
export type TxType =
  | { BridgeTokens: number } // TODO could use bigint for u64/U256-like values, as JavaScript's Number type is not suitable for very large integers.
  | { WithdrawTokens: number }
  | {
    Transfer: {
      from: string;
      to: string;
      amount: number;
    }
  }
  | {
    Mint: {
      to: string;
      amount: number;
    }
  };

export interface SequencerStore {
  balances: Record<string, number>
  set: (partial: SequencerStore | Partial<SequencerStore>) => void
}

const useSequencerStore = create<SequencerStore>()(
  persist(
    (set) => ({  // get
      balances: {},
      set,
    }),
    {
      name: 'sequencer', // unique name
      storage: createJSONStorage(() => localStorage), // (optional) by default, 'localStorage' is used
    }
  )
)

export default useSequencerStore
