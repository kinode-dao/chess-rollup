import { useEffect } from "react";
import useSequencerStore from "./store";

import Bridge from "./components/Bridge";
import InitiateWithdraw from "./components/InitiateWithdraw";
import Withdraw from "./components/Withdraw";
import NavBar from "./components/Navbar";
import ProposeGame from "./components/ProposeGame";
import Transfer from "./components/Transfer";
import MyGames from "./components/MyGames";
import PendingGames from "./components/PendingGames";

declare global {
  var window: Window & typeof globalThis;
  var our: { node: string; process: string };
}

const BASE_URL = import.meta.env.BASE_URL;
if (window.our) window.our.process = BASE_URL?.replace("/", "");

function App() {
  const { set } = useSequencerStore();

  useEffect(() => {
    console.log(`${BASE_URL}/rpc`)
    fetch(`${BASE_URL}/rpc`)
      .then((res) => res.json())
      .then((state) => {
        console.log('state', state);
        set({ ...state });
      })
      .catch(console.error);
  }, []);

  return (
    <>

      <div className="grid grid-cols-3 gap-4">
        <div className="col-span-3">
          <NavBar />
        </div>
        <div className="col-span-1">
          <div className="py-4 px-2">
            <Bridge />
          </div>
          <div className="py-4 px-2">
            <InitiateWithdraw baseUrl={BASE_URL} />
          </div>
          <div className="py-4 px-2">
            <Withdraw />
          </div>
          <div className="py-4 px-2">
            <Transfer baseUrl={BASE_URL} />
          </div>
          <div className="py-4 px-2">
            <ProposeGame baseUrl={BASE_URL} />
          </div>
          <div className="py-4 px-2">
            <PendingGames baseUrl={BASE_URL} />
          </div>
        </div>
        <div className="col-span-2">
          <MyGames baseUrl={BASE_URL} />
        </div>
      </div>
    </>
  );
}

export default App;
