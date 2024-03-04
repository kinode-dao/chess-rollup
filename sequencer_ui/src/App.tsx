import {
  FormEvent,
  useCallback,
  useEffect,
  useState,
} from "react";
// import UqbarEncryptorApi from "@uqbar/client-encryptor-api";
import useSequencerStore, { WrappedTransaction, TxType } from "./store";
import { ethers } from 'ethers';
// import "./App.css";

declare global {
  var window: Window & typeof globalThis;
  var our: { node: string; process: string };
}

const BASE_URL = import.meta.env.BASE_URL;
if (window.our) window.our.process = BASE_URL?.replace("/", "");

function App() {
  const { balances, set } = useSequencerStore();
  const [bridgeAmount, setBridgeAmount] = useState(0);

  // get balances
  useEffect(() => {
    // new UqbarEncryptorApi({
    //   uri: WEBSOCKET_URL,
    //   nodeId: window.our.node,
    //   processId: window.our.process,
    //   onMessage: handleWsMessage,
    // });

    fetch(`${BASE_URL}/rpc`)
      .then((res) => res.json())
      .then((balances) => {
        console.log('balances', balances);
        set({ balances });
      })
      .catch(console.error);
  }, []);

  // useEffect(() => {
  //   const initializeProvider = async () => {
  //     if (window.ethereum) {
  //       await window.ethereum.request({ method: 'eth_requestAccounts' });
  //       const provider = new ethers.providers.Web3Provider(window.ethereum);
  //       const accounts = await provider.send("eth_requestAccounts", []);
  //       setAccount(accounts[0]);
  //       setProvider(provider);
  //     }
  //   };

  //   initializeProvider();
  // }, []);


  const bridge = useCallback(
    async (e: FormEvent) => {
      e.preventDefault();
      if (!window.ethereum) {
        console.error('Ethereum wallet is not connected');
        return;
      }
      console.log('bridge', bridgeAmount);

      try {
        let tx: TxType = {
          BridgeTokens: bridgeAmount,
        }


        const accounts = await window.ethereum.request({ method: 'eth_requestAccounts' });
        const account = accounts[0];
        const signature = await window.ethereum.request({
          method: 'personal_sign',
          params: [JSON.stringify(tx), account],
        });
        console.log(`Message: ${tx}`);
        console.log(`Signature: ${signature}`);

        let wtx: WrappedTransaction = {
          pub_key: account.slice(2),
          sig: signature.slice(2),
          data: tx
        };

        const receipt = await fetch(`${BASE_URL}/rpc`, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify(wtx),
        });
        console.log('receipt', receipt);
        //     const createdGame = await fetch(`${BASE_URL}/rpc`, {
        //       method: "POST",
        //       headers: {
        //         "Content-Type": "application/json",
        //       },
        //       body: JSON.stringify({ id: newGame }),
        //     }).then((r) => {
        //       if (r.status === 409) {
        //         if (games[newGame]) {
        //           setScreen(newGame);
        //         } else {
        //           alert(
        //             "Game already exists, please refresh the page and select it."
        //           );
        //         }
        //         throw new Error("Game already exists");
        //       } else if (r.status === 503) {
        //         alert(
        //           `${newGame} may be offline, please confirm it is online and try again.`
        //         );
        //         throw new Error("Player offline");
        //       } else if (r.status === 400) {
        //         alert("Please enter a valid player ID");
        //         throw new Error("Invalid player ID");
        //       } else if (r.status > 399) {
        //         alert("There was an error creating the game. Please try again.");
        //         throw new Error("Error creating game");
        //       }

        //       return r.json();
        //     });

        //     const allGames = { ...games };
        //     allGames[createdGame.id] = createdGame;
        //     set({ games: allGames });
        //     setScreen(newGame);
        //     setNewGame("");
      } catch (err) {
        console.error(err);
      }
    },
    [balances, bridgeAmount, setBridgeAmount, set]
  );

  return (
    <div
      className="justify-center items-center"
    >
      <h4 className="m-4 row justify-center">
        Sequencer
      </h4>
      <div
        className="flex flex-col items-center"
      >
        <h4 className="m-2">Balances</h4>
        <div className="flex flex-col overflow-scroll">
          {Object.keys(balances).map((address, i) => (
            <p key={i}>{`${address}: ${balances[address]}`}</p>
          ))}
        </div>
      </div>
      <div
        className="flex flex-col items-center"
      >
        <h4 className="m-2">Bridge</h4>
        <div className="flex flex-col overflow-scroll">
          <form onSubmit={bridge}>
            <input
              type="number"
              value={bridgeAmount}
              onChange={(e) => setBridgeAmount(Number(e.target.value))}
            />
            <button type="submit">Bridge</button>
          </form>
        </div>
      </div>
    </div>
  );
}

export default App;
