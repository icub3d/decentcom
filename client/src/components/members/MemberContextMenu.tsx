import { useState } from "react";

import type { Member } from "../../api/members";
import { BAN_MEMBERS, KICK_MEMBERS, usePermissions } from "../../hooks/usePermissions";

interface MemberContextMenuProps {
  member: Member;
  onKick: (pubkey: string) => Promise<void>;
  onBan: (pubkey: string, reason?: string) => Promise<void>;
}

export function MemberContextMenu({ member, onKick, onBan }: MemberContextMenuProps) {
  const permissions = usePermissions();
  const canKick = permissions.has(KICK_MEMBERS);
  const canBan = permissions.has(BAN_MEMBERS);
  const [pending, setPending] = useState(false);

  async function handleKick() {
    if (!canKick || pending) {
      return;
    }
    setPending(true);
    try {
      await onKick(member.pubkey);
    } finally {
      setPending(false);
    }
  }

  async function handleBan() {
    if (!canBan || pending) {
      return;
    }
    setPending(true);
    try {
      await onBan(member.pubkey);
    } finally {
      setPending(false);
    }
  }

  return (
    <div className="flex gap-2">
      {canKick && (
        <button
          onClick={() => {
            void handleKick();
          }}
          disabled={pending}
          className="rounded-md border border-ctp-yellow/50 bg-ctp-yellow/10 px-2 py-1 text-xs text-ctp-yellow hover:bg-ctp-yellow/20 disabled:opacity-60"
        >
          Kick
        </button>
      )}
      {canBan && (
        <button
          onClick={() => {
            void handleBan();
          }}
          disabled={pending}
          className="rounded-md border border-ctp-red/50 bg-ctp-red/10 px-2 py-1 text-xs text-ctp-red hover:bg-ctp-red/20 disabled:opacity-60"
        >
          Ban
        </button>
      )}
    </div>
  );
}
