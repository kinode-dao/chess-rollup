import {
  FormEvent,
  useCallback,
  useEffect,
  useState,
} from "react";
// import UqbarEncryptorApi from "@uqbar/client-encryptor-api";
import useSequencerStore, { TxType, WrappedTransaction } from "./store";
import { ethers, BigNumber } from "ethers";
import sendTx from "./tx";
import { useWeb3React } from '@web3-react/core'
import { ConnectionType } from './libs/connections'
import { ConnectionOptions } from './libs/components/ConnectionOptions'

declare global {
  var window: Window & typeof globalThis;
  var our: { node: string; process: string };
}

const BASE_URL = import.meta.env.BASE_URL;
if (window.our) window.our.process = BASE_URL?.replace("/", "");

function App() {
  const { chainId, account, isActive, provider } = useWeb3React()
  const [connectionType, setConnectionType] = useState<ConnectionType | null>(null)

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
        console.log('state', state);
        set({ ...state });
      })
      .catch(console.error);
  }, []);

  const transfer = useCallback(
    async (e: FormEvent) => {
      e.preventDefault();
      if (!account || !provider) {
        console.log('account', account)
        console.log("provider", provider)
        console.error('Ethereum wallet is not connected');
        return;
      }

      try {
        let tx: TxType = {
          Transfer: {
            from: account.toLowerCase(),
            to: transferTo.toLowerCase(),
            amount: BigNumber.from(transferAmount).toHexString().replace(/^0x0+/, '0x'), // for some reason there's a leading zero...really annoying!
          },
        }

        const signature = await provider.getSigner().signMessage(JSON.stringify(tx));
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
    [account, provider, balances, transferAmount, transferTo, setTransferAmount, setTransferTo, set]
  );

  const proposeGame = useCallback(
    async (e: FormEvent) => {
      e.preventDefault();
      try {
        if (!account || !provider) {
          window.alert('Ethereum wallet is not connected');
          return;
        }
        let tx: TxType = {
          ProposeGame: {
            white: account.toLowerCase(),
            black: requestTo.toLowerCase(),
            wager: BigNumber.from(wagerAmount).toHexString().replace(/^0x0+/, '0x'), // for some reason there's a leading zero...really annoying!
          },
        }

        const signature = await provider.getSigner().signMessage(JSON.stringify(tx));
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
    [account, provider, balances, transferAmount, transferTo, setTransferAmount, setTransferTo, set]
  );

  return (
    <div
      className="justify-center items-center"
    >
      <h4 className="m-4 row justify-center">
        Chess Sequencer
      </h4>
      <h4 className="m-4 row justify-center">
        {account ? `${account}` : "no wallet connected"}
      </h4>
      <div
        className="flex flex-col items-center"
      >
        <ConnectionOptions
          activeConnectionType={connectionType}
          isConnectionActive={isActive}
          onActivate={setConnectionType}
          onDeactivate={setConnectionType}
        />
      </div>
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
            const { white, black, wager } = pending_games[gameId]; // accepted
            if (account == black) {
              return (
                <div key={i}>
                  <p>{`You have been challenged by ${white} for ${BigNumber.from(wager)} WEI`}</p>
                  <button onClick={() => {
                    let tx: TxType = {
                      StartGame: gameId,
                    }
                    sendTx(tx, account, `${BASE_URL}/rpc`);
                  }}>Accept</button>
                </div>
              )
            } else {
              return <p key={i}>{`${gameId}: ${white} vs ${black} for ${BigNumber.from(wager)} WEI`}</p>
            }
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
            <input type="text" placeholder="opponent" value={requestTo} onChange={(e) => setRequestTo(e.target.value)} />
            <input
              type="number"
              value={wagerAmount}
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
