import {
  FormEvent,
  useCallback,
  useEffect,
  useState,
} from "react";
// import UqbarEncryptorApi from "@uqbar/client-encryptor-api";
import useSequencerStore, { WrappedTransaction, TxType } from "./store";
import { ethers, BigNumber } from "ethers";

declare global {
  var window: Window & typeof globalThis;
  var our: { node: string; process: string };
}

const BASE_URL = import.meta.env.BASE_URL;
if (window.our) window.our.process = BASE_URL?.replace("/", "");

function App() {
  const { balances, set } = useSequencerStore();
  const [transferTo, setTransferTo] = useState('0x95222290dd7278aa3ddd389cc1e1d165cc4bafe5');
  const [transferAmount, setTransferAmount] = useState(4);
  // const [mintTo, setMintTo] = useState('0x95222290dd7278aa3ddd389cc1e1d165cc4bafe5');
  // const [mintAmount, setMintAmount] = useState(5);

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

  const transfer = useCallback(
    async (e: FormEvent) => {
      e.preventDefault();
      if (!window.ethereum) {
        console.error('Ethereum wallet is not connected');
        return;
      }

      try {
        const accounts = await window.ethereum.request({ method: 'eth_requestAccounts' });
        const account = accounts[0];

        let tx: TxType = {
          Transfer: {
            from: account.toLowerCase(),
            to: transferTo.toLowerCase(),
            amount: BigNumber.from(transferAmount).toHexString().replace(/^0x0+/, '0x'), // for some reason there's a leading zero...really annoying!
          },
        }


        const signature = await window.ethereum.request({
          method: 'personal_sign',
          params: [JSON.stringify(tx), account],
        });
        const { v, r, s } = ethers.utils.splitSignature(signature);

        let wtx: WrappedTransaction = {
          pub_key: account,
          sig: {
            r, s, v
          },
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
      } catch (err) {
        console.error(err);
      }
    },
    [balances, transferAmount, transferTo, setTransferAmount, setTransferTo, set]
  );

  // const mint = useCallback(
  //   async (e: FormEvent) => {
  //     e.preventDefault();
  //     if (!window.ethereum) {
  //       console.error('Ethereum wallet is not connected');
  //       return;
  //     }

  //     try {
  //       const accounts = await window.ethereum.request({ method: 'eth_requestAccounts' });
  //       const account = accounts[0];

  //       let tx: TxType = {
  //         Mint: {
  //           to: mintTo.toLowerCase(),
  //           amount: BigNumber.from(mintAmount).toHexString().replace(/^0x0+/, '0x'), // for some reason there's a leading zero...really annoying!
  //         },
  //       }

  //       const signature = await window.ethereum.request({
  //         method: 'personal_sign',
  //         params: [JSON.stringify(tx), account],
  //       });
  //       const { r, s, v } = ethers.utils.splitSignature(signature);

  //       let wtx: WrappedTransaction = {
  //         pub_key: account,
  //         sig: {
  //           r,
  //           s,
  //           v,
  //         },
  //         data: tx
  //       };

  //       const receipt = await fetch(`${BASE_URL}/rpc`, {
  //         method: "POST",
  //         headers: {
  //           "Content-Type": "application/json",
  //         },
  //         body: JSON.stringify(wtx),
  //       });
  //       console.log('receipt', receipt);
  //     } catch (err) {
  //       console.error(err);
  //     }
  //   },
  //   [balances, mintAmount, mintTo, setMintAmount, setMintTo, set]
  // );

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
            <p key={i}>{`${address}: ${BigNumber.from(balances[address])} WEI`}</p>
          ))}
        </div>
      </div>
      <div
        className="flex flex-col items-center"
      >
        <h4 className="m-2">Transfer</h4>
        <div className="flex flex-col overflow-scroll">
          <form onSubmit={transfer}>
            <input type="text" placeholder="to" value={transferTo} onChange={(e) => setTransferTo(e.target.value)} />
            <input
              type="number"
              value={transferAmount}
              onChange={(e) => setTransferAmount(Number(e.target.value))}
            />
            <button type="submit">Transfer</button>
          </form>
        </div>
        {/*  */}
        {/* <br /><br /><br />
        <h4 className="m-2">Mint</h4>
        <div className="flex flex-col overflow-scroll">
          <form onSubmit={mint}>
            <input type="text" placeholder="to" value={mintTo} onChange={(e) => setMintTo(e.target.value)} />
            <input
              type="number"
              value={mintAmount}
              onChange={(e) => setMintAmount(Number(e.target.value))}
            />
            <button type="submit">Mint</button>
          </form>
        </div> */}
      </div>
    </div>
  );
}

export default App;
