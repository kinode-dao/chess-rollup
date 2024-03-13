import {
  FormEvent,
  useCallback,
  useEffect,
  useState,
} from "react";
// import UqbarEncryptorApi from "@uqbar/client-encryptor-api";
import useSequencerStore, { WrappedTransaction, TxType } from "./store";
import { ethers, BigNumber } from "ethers";
import sendTx from "./tx";
import { useWeb3React } from '@web3-react/core'

declare global {
  var window: Window & typeof globalThis;
  var our: { node: string; process: string };
}

const BASE_URL = import.meta.env.BASE_URL;
if (window.our) window.our.process = BASE_URL?.replace("/", "");

function App() {
  const { chainId, account, isActive } = useWeb3React()
  const { balances, pending_games, games, set } = useSequencerStore();
  const [transferTo, setTransferTo] = useState('0x95222290dd7278aa3ddd389cc1e1d165cc4bafe5');
  const [transferAmount, setTransferAmount] = useState(4);

  const [requestTo, setRequestTo] = useState('0x95222290dd7278aa3ddd389cc1e1d165cc4bafe5');
  const [wagerAmount, setWagerAmount] = useState(4);

  // get balances
  useEffect(() => {
    // new UqbarEncryptorApi({
    //   uri: WEBSOCKET_URL,
    //   nodeId: window.our.node,
    //   processId: window.our.process,
    //   onMessage: handleWsMessage,
    // });
    console.log(`${BASE_URL}/rpc`)
    fetch(`${BASE_URL}/rpc`)
      .then((res) => res.json())
      .then((state) => {
        console.log(state);
        set({ ...state });
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

        await sendTx(tx, account, `${BASE_URL}/rpc`);
      } catch (err) {
        console.error(err);
      }
    },
    [balances, transferAmount, transferTo, setTransferAmount, setTransferTo, set]
  );

  const proposeGame = useCallback(
    async (e: FormEvent) => {
      e.preventDefault();
      try {
        if (!account) {
          window.alert('Ethereum wallet is not connected');
          return;
        }
        let tx: TxType = {
          ProposeGame: {
            white: account,
            black: requestTo.toLowerCase(),
            wager: BigNumber.from(wagerAmount).toHexString().replace(/^0x0+/, '0x'), // for some reason there's a leading zero...really annoying!
          },
        }

        await sendTx(tx, account, `${BASE_URL}/rpc`);
      } catch (err) {
        console.error(err);
      }
    },
    [balances, transferAmount, transferTo, setTransferAmount, setTransferTo, set]
  );

  return (
    <div
      className="justify-center items-center"
    >
      <h4 className="m-4 row justify-center">
        {`Sequencer; connected wallet: ${account}; chainId: ${chainId}; active: ${isActive}`}
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
        <h4 className="m-2">Pending Games</h4>
        <div className="flex flex-col overflow-scroll">
          {Object.keys(pending_games).map((gameId, i) => {
            console.log(pending_games[gameId]);
            const { white, black, wager } = pending_games[gameId]; // accepted
            return <p key={i}>{`${gameId}: ${white} vs ${black} for ${BigNumber.from(wager)} WEI`}</p>
          })}
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
      </div>
      {/*  */}
      <div
        className="flex flex-col items-center"
      >
        <h4 className="m-2">Propose Game</h4>
        <div className="flex flex-col overflow-scroll">
          <form onSubmit={proposeGame}>
            <input type="text" placeholder="opponent" value={transferTo} onChange={(e) => setRequestTo(e.target.value)} />
            <input
              type="number"
              value={transferAmount}
              onChange={(e) => setWagerAmount(Number(e.target.value))}
            />
            <button type="submit">Transfer</button>
          </form>
        </div>
      </div>
    </div>
  );
}

export default App;
