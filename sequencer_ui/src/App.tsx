import { useEffect } from "react";
import useSequencerStore from "./store";

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
    <div
      className="justify-center items-center"
    >
      <NavBar />
      <PendingGames baseUrl={BASE_URL} />
      <MyGames baseUrl={BASE_URL} />
      <Transfer baseUrl={BASE_URL} />
      <ProposeGame baseUrl={BASE_URL} />
    </div>
  );
}

export default App;
