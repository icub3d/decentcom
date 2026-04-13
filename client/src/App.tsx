import { useState } from "react";

import { ServerConnect } from "./components/connection/ServerConnect";
import { AppShell } from "./components/layout/AppShell";
import { useIdentity } from "./hooks/useIdentity";
import { useAppStore } from "./stores/appStore";
import { useServerStore } from "./stores/serverStore";
import { Setup } from "./pages/Setup";
import "./App.css";

function App() {
  const {
    hasIdentity,
    loading,
    error,
    generateIdentity,
    importIdentity,
    refresh,
  } = useIdentity();
  const { currentServerId, addServer } = useAppStore();
  const { connect, status } = useServerStore();
  const [connectLoading, setConnectLoading] = useState(false);
  const [connectError, setConnectError] = useState<string | null>(null);

  async function handleConnect(address: string) {
    setConnectLoading(true);
    setConnectError(null);
    try {
      await connect(address);
      addServer(address);
    } catch (err) {
      setConnectError(err instanceof Error ? err.message : String(err));
    } finally {
      setConnectLoading(false);
    }
  }

  if (loading && hasIdentity === null) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-slate-900 text-slate-100">
        <div className="animate-pulse">Loading identity...</div>
      </div>
    );
  }

  if (!hasIdentity) {
    return (
      <main className="min-h-screen flex items-center justify-center bg-slate-900 p-4">
        <Setup 
          onGenerate={generateIdentity} 
          onImport={importIdentity} 
          onComplete={refresh} 
        />
      </main>
    );
  }

  if (!currentServerId) {
    return (
      <ServerConnect
        loading={connectLoading || status === "connecting"}
        error={connectError}
        onConnect={handleConnect}
      />
    );
  }

  return (
    <>
      {error && (
        <div className="fixed right-4 top-4 z-20 rounded-lg border border-rose-700 bg-rose-900/40 px-4 py-2 text-sm text-rose-300">
          {error}
        </div>
      )}
      <AppShell />
    </>
  );
}

export default App;
