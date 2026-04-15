import { useState } from "react";

import type { AllowlistEntry } from "../../api/members";

interface AllowlistEditorProps {
  entries: AllowlistEntry[];
  onAdd: (pubkey: string) => Promise<void>;
  onRemove: (pubkey: string) => Promise<void>;
}

export function AllowlistEditor({ entries, onAdd, onRemove }: AllowlistEditorProps) {
  const [pubkey, setPubkey] = useState("");

  return (
    <section className="rounded-xl border border-ctp-overlay0 bg-ctp-mantle/90 p-3">
      <h3 className="mb-3 text-sm font-bold text-ctp-text">Allowlist</h3>
      <div className="mb-3 flex gap-2">
        <input
          value={pubkey}
          onChange={(event) => setPubkey(event.target.value)}
          placeholder="Ed25519 public key"
          className="flex-1 rounded-lg border border-ctp-overlay0 bg-ctp-base px-3 py-2 text-xs text-ctp-text"
        />
        <button
          onClick={() => {
            if (pubkey.trim().length === 0) {
              return;
            }
            void onAdd(pubkey.trim());
            setPubkey("");
          }}
          className="rounded-lg bg-ctp-blue px-3 py-2 text-xs font-bold text-ctp-crust"
        >
          Add
        </button>
      </div>
      <div className="max-h-64 space-y-2 overflow-auto pr-1">
        {entries.map((entry) => (
          <div
            key={entry.pubkey}
            className="flex items-center justify-between rounded-lg border border-ctp-surface1 bg-ctp-base/70 p-2"
          >
            <span className="text-xs text-ctp-text">{entry.pubkey}</span>
            <button
              onClick={() => {
                void onRemove(entry.pubkey);
              }}
              className="rounded-md border border-ctp-red/40 bg-ctp-red/10 px-2 py-1 text-xs text-ctp-red"
            >
              Remove
            </button>
          </div>
        ))}
      </div>
    </section>
  );
}
