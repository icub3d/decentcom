import type { BotApproval } from "../../api/members";

interface BotApprovalPanelProps {
  pendingBots: BotApproval[];
  onApprove: (pubkey: string) => Promise<void>;
  onRevoke: (pubkey: string) => Promise<void>;
}

function truncatePubkey(pubkey: string): string {
  if (pubkey.length <= 14) return pubkey;
  return `${pubkey.slice(0, 8)}...${pubkey.slice(-6)}`;
}

export function BotApprovalPanel({ pendingBots, onApprove, onRevoke }: BotApprovalPanelProps) {
  return (
    <section className="rounded-xl border border-ctp-overlay0 bg-ctp-mantle/90 p-3">
      <h3 className="mb-3 text-sm font-bold text-ctp-text">Pending Bots</h3>
      {pendingBots.length === 0 ? (
        <p className="text-xs text-ctp-subtext0">No bots awaiting approval.</p>
      ) : (
        <div className="max-h-64 space-y-2 overflow-auto pr-1">
          {pendingBots.map((bot) => (
            <div
              key={bot.pubkey}
              className="flex items-center justify-between rounded-lg border border-ctp-surface1 bg-ctp-base/70 p-2"
            >
              <div>
                <div className="flex items-center gap-1">
                  <span className="text-xs font-semibold text-ctp-text">
                    {truncatePubkey(bot.pubkey)}
                  </span>
                  <span className="rounded bg-ctp-yellow/20 px-1 py-0.5 text-[10px] font-bold text-ctp-yellow">
                    PENDING
                  </span>
                </div>
              </div>
              <div className="flex gap-1">
                <button
                  onClick={() => { void onApprove(bot.pubkey); }}
                  className="rounded-md border border-ctp-green/40 bg-ctp-green/10 px-2 py-1 text-xs text-ctp-green hover:bg-ctp-green/20"
                >
                  Approve
                </button>
                <button
                  onClick={() => { void onRevoke(bot.pubkey); }}
                  className="rounded-md border border-ctp-red/40 bg-ctp-red/10 px-2 py-1 text-xs text-ctp-red hover:bg-ctp-red/20"
                >
                  Reject
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}
