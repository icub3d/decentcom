import { useIdentity } from "./hooks/useIdentity";
import { Setup } from "./pages/Setup";
import "./App.css";

function App() {
  const { 
    hasIdentity, 
    publicKey, 
    loading, 
    error, 
    generateIdentity, 
    importIdentity, 
    refresh 
  } = useIdentity();

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

  return (
    <main className="min-h-screen flex flex-col items-center justify-center bg-slate-900 text-slate-100 p-8">
      <div className="max-w-2xl w-full text-center space-y-8">
        <div className="space-y-2">
          <h1 className="text-4xl font-black tracking-tight text-blue-500">decentcom</h1>
          <p className="text-slate-400 font-medium">Decentralized, self-hostable communication.</p>
        </div>
        
        <div className="bg-slate-800 p-6 rounded-2xl border border-slate-700 shadow-2xl space-y-4">
          <h2 className="text-sm font-bold uppercase tracking-widest text-slate-500">Your Identity</h2>
          <div className="bg-slate-900 p-4 rounded-xl border border-slate-700 break-all font-mono text-blue-400">
            {publicKey}
          </div>
          <p className="text-xs text-slate-500">
            This public key is your unique ID across all decentcom servers.
          </p>
        </div>

        {error && (
          <div className="p-4 bg-red-900/20 border border-red-800 rounded-xl text-red-400 text-sm">
            {error}
          </div>
        )}
      </div>
    </main>
  );
}

export default App;
