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
    <main className="min-h-screen bg-ctp-base text-ctp-text flex items-center justify-center p-6">
      <form
        onSubmit={handleSubmit}
        className="w-full max-w-lg rounded-2xl border border-ctp-overlay0 bg-ctp-mantle/90 p-8 shadow-2xl space-y-6"
      >
        <header className="space-y-2">
          <h1 className="text-3xl font-black tracking-tight text-ctp-blue">Connect to a Server</h1>
          <p className="text-ctp-subtext0">
            Enter your decentcom server address to authenticate and join channels.
          </p>
        </header>

        <div className="space-y-2">
          <label htmlFor="server-address" className="text-sm font-semibold text-ctp-subtext1">
            Server URL
          </label>
          <input
            id="server-address"
            value={address}
            onChange={(e) => setAddress(e.target.value)}
            className="w-full rounded-xl border border-ctp-overlay0 bg-ctp-base px-4 py-3 text-ctp-text focus:outline-none focus:ring-2 focus:ring-ctp-blue"
            placeholder="http://127.0.0.1:8080"
          />
        </div>

        {(localError || error) && (
          <p className="rounded-lg border border-ctp-red bg-ctp-red/20 p-3 text-sm text-ctp-red">
            {localError ?? error}
          </p>
        )}

        <button
          type="submit"
          disabled={loading}
          className="w-full rounded-xl bg-ctp-blue px-4 py-3 text-lg font-bold text-ctp-crust transition hover:bg-ctp-sapphire disabled:opacity-60"
        >
          {loading ? "Connecting..." : "Connect"}
        </button>
      </form>
    </main>
  );
}
