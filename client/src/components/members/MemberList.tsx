import type { Member } from "../../api/members";
import { MemberContextMenu } from "./MemberContextMenu";

interface MemberListProps {
  members: Member[];
  onKick: (pubkey: string) => Promise<void>;
  onBan: (pubkey: string, reason?: string) => Promise<void>;
}

function truncatePubkey(pubkey: string): string {
  if (pubkey.length <= 14) {
    return pubkey;
  }
  return `${pubkey.slice(0, 8)}...${pubkey.slice(-6)}`;
}

export function MemberList({ members, onKick, onBan }: MemberListProps) {
  const sorted = [...members].sort((a, b) => {
    const roleA = a.roles[0]?.position ?? 0;
    const roleB = b.roles[0]?.position ?? 0;
    return roleB - roleA || a.pubkey.localeCompare(b.pubkey);
  });

  return (
    <section className="rounded-xl border border-ctp-overlay0 bg-ctp-mantle/90 p-3">
      <div className="mb-3 flex items-center justify-between">
        <h3 className="text-sm font-bold text-ctp-text">Members</h3>
        <span className="text-xs text-ctp-subtext0">{sorted.length}</span>
      </div>
      <div className="max-h-72 space-y-2 overflow-auto pr-1">
        {sorted.map((member) => (
          <div
            key={member.user_id}
            className="rounded-lg border border-ctp-surface1 bg-ctp-base/70 p-2"
          >
            <div className="flex items-center justify-between gap-2">
              <div>
                <div className="text-sm font-semibold text-ctp-text">{truncatePubkey(member.pubkey)}</div>
                <div className="text-xs text-ctp-subtext0">
                  Joined {new Date(member.joined_at).toLocaleDateString()}
                </div>
              </div>
              <MemberContextMenu member={member} onKick={onKick} onBan={onBan} />
            </div>
            <div className="mt-2 flex flex-wrap gap-1">
              {member.roles.map((role) => (
                <span
                  key={role.id}
                  className="rounded-md border border-ctp-overlay0 bg-ctp-surface0 px-2 py-0.5 text-xs text-ctp-subtext1"
                >
                  {role.name}
                </span>
              ))}
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}
