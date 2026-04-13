import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [pong, setPong] = useState<string>("");

  useEffect(() => {
    invoke<string>("ping")
      .then(setPong)
      .catch(() => setPong("error"));
  }, []);

  return (
    <main className="min-h-screen flex items-center justify-center bg-slate-900 text-slate-100">
      <div className="text-center space-y-2">
        <h1 className="text-2xl font-semibold">decentcom</h1>
        <p className="text-sm opacity-75">
          Tauri bridge: <span data-testid="ping-result">{pong || "..."}</span>
        </p>
      </div>
    </main>
  );
}

export default App;
