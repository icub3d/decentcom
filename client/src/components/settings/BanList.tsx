import type { Ban } from "../../api/members";

interface BanListProps {
  bans: Ban[];
  onUnban: (pubkey: string) => Promise<void>;
}

export function BanList({ bans, onUnban }: BanListProps) {
  return (
    <section className="rounded-xl border border-ctp-overlay0 bg-ctp-mantle/90 p-3">
      <h3 className="mb-3 text-sm font-bold text-ctp-text">Bans</h3>
      <div className="max-h-64 space-y-2 overflow-auto pr-1">
        {bans.map((ban) => (
          <div
            key={ban.pubkey}
            className="flex items-center justify-between rounded-lg border border-ctp-surface1 bg-ctp-base/70 p-2"
          >
            <div>
              <div className="text-xs font-semibold text-ctp-text">{ban.pubkey}</div>
              <div className="text-xs text-ctp-subtext0">{ban.reason ?? "No reason"}</div>
            </div>
            <button
              onClick={() => {
                void onUnban(ban.pubkey);
              }}
              className="rounded-md border border-ctp-green/40 bg-ctp-green/10 px-2 py-1 text-xs text-ctp-green hover:bg-ctp-green/20"
            >
              Unban
            </button>
          </div>
        ))}
      </div>
    </section>
  );
}
