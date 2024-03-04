import { create } from 'zustand'
import { persist, createJSONStorage } from 'zustand/middleware'

export interface WrappedTransaction {
  pub_key: string; // Converted camelCase for TypeScript conventions
  sig: string;
  data: TxType; // Still a hex string, but consider using ArrayBuffer or similar for binary data handling in JS/TS
  // Additional fields like nonces, value, gas, gasPrice, gasLimit, etc., can be added as needed.
}

// For the `TxType` enum, TypeScript uses a combination of types and interfaces to achieve similar functionality.
export type TxType =
  | { BridgeTokens: number } // TODO could use bigint for u64/U256-like values, as JavaScript's Number type is not suitable for very large integers.
  | { WithdrawTokens: number }
  | {
    Transfer: {
      from: string;
      to: string;
      amount: bigint;
    }
  }
  | {
    Mint: {
      to: string;
      amount: bigint;
    }
  };

export interface SequencerStore {
  balances: Record<string, number>
  handleWsMessage: (message: string) => void
  set: (partial: SequencerStore | Partial<SequencerStore>) => void
}

const useSequencerStore = create<SequencerStore>()(
  persist(
    (set, get) => ({
      balances: {},
      handleWsMessage: (json: string | Blob) => {

        // if (typeof json === 'string') {
        //   try {
        //     const { kind, data } = JSON.parse(json) as WsMessage;
        //     if (kind === 'game_update') {
        //       set({ games: { ...get().games, [data.id]: data } })
        //     }
        //   } catch (error) {
        //     console.error("Error parsing WebSocket message", error);
        //   }
        // } else {
        //   const reader = new FileReader();

        //   reader.onload = function (event) {
        //     if (typeof event?.target?.result === 'string') {
        //       try {
        //         const { kind, data } = JSON.parse(event.target.result) as WsMessage;

        //         if (kind === 'game_update') {
        //           set({ games: { ...get().games, [data.id]: data } })
        //         }
        //       } catch (error) {
        //         console.error("Error parsing WebSocket message", error);
        //       }
        //     }
        //   };

        //   reader.readAsText(json);
        // }
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
