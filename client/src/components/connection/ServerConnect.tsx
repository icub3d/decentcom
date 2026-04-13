import { FormEvent, useState } from "react";

interface ServerConnectProps {
  loading: boolean;
  error: string | null;
  onConnect: (address: string) => Promise<void>;
}

export function ServerConnect({ loading, error, onConnect }: ServerConnectProps) {
  const [address, setAddress] = useState("http://127.0.0.1:8080");
  const [localError, setLocalError] = useState<string | null>(null);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const trimmed = address.trim();
    if (!trimmed) {
      setLocalError("Server address is required");
      return;
    }
    setLocalError(null);
    await onConnect(trimmed);
  }

  return (
    <main className="min-h-screen bg-slate-900 text-slate-100 flex items-center justify-center p-6">
      <form
        onSubmit={handleSubmit}
        className="w-full max-w-lg rounded-2xl border border-slate-700 bg-slate-800/80 p-8 shadow-2xl space-y-6"
      >
        <header className="space-y-2">
          <h1 className="text-3xl font-black tracking-tight text-blue-400">Connect to a Server</h1>
          <p className="text-slate-400">
            Enter your decentcom server address to authenticate and join channels.
          </p>
        </header>

        <div className="space-y-2">
          <label htmlFor="server-address" className="text-sm font-semibold text-slate-300">
            Server URL
          </label>
          <input
            id="server-address"
            value={address}
            onChange={(e) => setAddress(e.target.value)}
            className="w-full rounded-xl border border-slate-700 bg-slate-900 px-4 py-3 text-slate-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
            placeholder="http://127.0.0.1:8080"
          />
        </div>

        {(localError || error) && (
          <p className="rounded-lg border border-rose-700 bg-rose-900/30 p-3 text-sm text-rose-300">
            {localError ?? error}
          </p>
        )}

        <button
          type="submit"
          disabled={loading}
          className="w-full rounded-xl bg-blue-600 px-4 py-3 text-lg font-bold text-white transition hover:bg-blue-500 disabled:opacity-60"
        >
          {loading ? "Connecting..." : "Connect"}
        </button>
      </form>
    </main>
  );
}
