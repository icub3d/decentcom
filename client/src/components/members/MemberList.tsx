import type { Member } from "../../api/members";
import type { Role } from "../../api/roles";
import { MemberContextMenu } from "./MemberContextMenu";
import { Avatar } from "../profile/Avatar";

interface MemberListProps {
  members: Member[];
  roles: Role[];
  memberRoleIdsByUserId: Record<string, string[]>;
  onKick: (pubkey: string) => Promise<void>;
  onBan: (pubkey: string, reason?: string) => Promise<void>;
  onAssignRole: (pubkey: string, roleId: string) => Promise<void>;
  onRemoveRole: (pubkey: string, roleId: string) => Promise<void>;
}

function truncatePubkey(pubkey: string): string {
  if (pubkey.length <= 14) {
    return pubkey;
  }
  return `${pubkey.slice(0, 8)}...${pubkey.slice(-6)}`;
}

export function MemberList({
  members,
  roles,
  memberRoleIdsByUserId,
  onKick,
  onBan,
  onAssignRole,
  onRemoveRole,
}: MemberListProps) {
  const sorted = [...members].sort((a, b) => {
    const roleA = a.roles[0]?.position ?? 0;
    const roleB = b.roles[0]?.position ?? 0;
    return roleB - roleA || a.pubkey.localeCompare(b.pubkey);
  });

  return (
    <section className="p-3">
      <div className="mb-3 flex items-center justify-between">
        <h3 className="text-sm font-bold text-ctp-text">Members</h3>
        <span className="text-xs text-ctp-subtext0">{sorted.length}</span>
      </div>
      <div className="space-y-2">
        {sorted.map((member) => (
          <div
            key={member.user_id}
            className="rounded-lg border border-ctp-surface1 bg-ctp-base/70 p-2"
          >
            <div className="flex items-center justify-between gap-2">
              <div className="flex items-center gap-2 min-w-0">
                <Avatar pubkey={member.pubkey} avatarHash={member.avatar_hash} size={28} />
                <div className="min-w-0">
                  <div className="flex items-center gap-1">
                    <span className="text-sm font-semibold text-ctp-text truncate">
                      {member.display_name ?? truncatePubkey(member.pubkey)}
                    </span>
                    {member.is_bot && (
                      <span className="shrink-0 rounded bg-ctp-blue/20 px-1 py-0.5 text-[10px] font-bold text-ctp-blue">
                        BOT
                      </span>
                    )}
                  </div>
                  {member.display_name && (
                    <div className="text-xs text-ctp-subtext0 truncate">{truncatePubkey(member.pubkey)}</div>
                  )}
                </div>
              </div>
              <MemberContextMenu
                member={member}
                roles={roles}
                memberRoleIds={memberRoleIdsByUserId[member.user_id] ?? []}
                onKick={onKick}
                onBan={onBan}
                onAssignRole={onAssignRole}
                onRemoveRole={onRemoveRole}
              />
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
