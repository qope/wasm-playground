import React, { useState } from "react";
import "./App.css";
import init, {
  ip_get,
  merkle_tree,
  zk_hash,
  make_lamport_sig,
  recursive_proof,
} from "wasm-lib";

function App() {
  const [result, setResult] = useState("");
  const [time, setTime] = useState(0);
  const [isLoading, setIsLoading] = useState(false);

  const doFunc = (f: Function) => {
    (async () => {
      setResult("");
      setTime(0);
      setIsLoading(true);
      const start = performance.now();
      await init();
      const r = await f();
      const end = performance.now();
      setResult(r);
      setTime(Math.round(end - start));
      setIsLoading(false);
    })();
  };

  return (
    <div className="App">
      <p>
        <button onClick={() => doFunc(ip_get)} disabled={isLoading}>
          get ip
        </button>
        <button onClick={() => doFunc(merkle_tree)} disabled={isLoading}>
          merkle tree
        </button>
        <button onClick={() => doFunc(zk_hash)} disabled={isLoading}>
          zk hash
        </button>
        <button onClick={() => doFunc(make_lamport_sig)} disabled={isLoading}>
          lamport sig
        </button>
        <button onClick={() => doFunc(recursive_proof)} disabled={isLoading}>
          recursive_proof
        </button>
      </p>
      <form>
        <input type="text" value={result} readOnly></input>
      </form>
      <p>time: {time} ms</p>
    </div>
  );
}

export default App;
