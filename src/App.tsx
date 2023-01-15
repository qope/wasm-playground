import React, { useState } from "react";
import "./App.css";
import init, { ip_get } from "wasm-lib";

function App() {
  const [result, setResult] = useState("");
  const [time, setTime] = useState(0);

  const getIP = () => {
    (async () => {
      setResult("");
      setTime(0);
      const start = performance.now();
      await init();
      const ip = await ip_get();
      console.log(ip);
      const end = performance.now();
      setResult(ip);
      setTime(Math.round(end - start));
    })();
  };

  return (
    <div className="App">
      <p>
        <button onClick={getIP}> get ip</button>
        {/* <button onClick={getIP}> get ip</button> */}
      </p>
      <p>{result}</p>
      <p>time: {time} ms</p>
    </div>
  );
}

export default App;
