import type { Invite } from "../../api/invites";

interface InviteListProps {
  invites: Invite[];
  loading?: boolean;
  onRevoke: (code: string) => Promise<void>;
}

export function InviteList({ invites, loading = false, onRevoke }: InviteListProps) {
  return (
    <section className="rounded-xl border border-ctp-overlay0 bg-ctp-mantle/80 p-4 space-y-3">
      <h3 className="text-sm font-bold text-ctp-text">Active Invites</h3>
      {loading ? (
        <p className="text-xs text-ctp-subtext0">Loading invites...</p>
      ) : invites.length === 0 ? (
        <p className="text-xs text-ctp-subtext0">No active invites.</p>
      ) : (
        <div className="space-y-2">
          {invites.map((invite) => (
            <div
              key={invite.code}
              className="rounded-lg border border-ctp-overlay0 bg-ctp-base p-3 text-xs text-ctp-subtext1"
            >
              <div className="font-semibold text-ctp-text">{invite.code}</div>
              <div>Uses: {invite.use_count} / {invite.max_uses === 0 ? "unlimited" : invite.max_uses}</div>
              <div>Expires: {invite.expires_at ?? "never"}</div>
              <button
                onClick={() => {
                  void onRevoke(invite.code);
                }}
                className="mt-2 rounded-md bg-ctp-red/20 px-2 py-1 font-semibold text-ctp-red"
              >
                Revoke
              </button>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}
