import React from "react";
import "./App.css";
import init, { ip_get } from "wasm-lib";

function App() {
  const getIP = () => {
    (async () => {
      await init();
      const ip = await ip_get();
      console.log(ip);
    })();
  };

  return (
    <div className="App">
      <p>
        <button onClick={getIP}> get ip</button>
      </p>
    </div>
  );
}

export default App;
