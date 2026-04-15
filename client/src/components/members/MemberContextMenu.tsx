import { useEffect, useRef, useState } from "react";

import type { Member } from "../../api/members";
import type { Role } from "../../api/roles";
import { BAN_MEMBERS, KICK_MEMBERS, MANAGE_ROLES, usePermissions } from "../../hooks/usePermissions";

interface MemberContextMenuProps {
  member: Member;
  roles: Role[];
  memberRoleIds: string[];
  onKick: (pubkey: string) => Promise<void>;
  onBan: (pubkey: string, reason?: string) => Promise<void>;
  onAssignRole: (pubkey: string, roleId: string) => Promise<void>;
  onRemoveRole: (pubkey: string, roleId: string) => Promise<void>;
}

type MenuView = "main" | "kick-confirm" | "ban-confirm";

export function MemberContextMenu({
  member,
  roles,
  memberRoleIds,
  onKick,
  onBan,
  onAssignRole,
  onRemoveRole,
}: MemberContextMenuProps) {
  const permissions = usePermissions();
  const canKick = permissions.has(KICK_MEMBERS);
  const canBan = permissions.has(BAN_MEMBERS);
  const canManageRoles = permissions.has(MANAGE_ROLES);

  const [open, setOpen] = useState(false);
  const [view, setView] = useState<MenuView>("main");
  const [pending, setPending] = useState(false);
  const [banReason, setBanReason] = useState("");
  const [pendingRoleId, setPendingRoleId] = useState<string | null>(null);
  const menuRef = useRef<HTMLDivElement>(null);

  const hasAnyPermission = canKick || canBan || canManageRoles;

  // Non-builtin, non-everyone roles available for assignment
  const assignableRoles = roles.filter(
    (r) => !r.is_builtin && r.name !== "@everyone",
  );

  // Close on outside click
  useEffect(() => {
    if (!open) return;
    function handleClickOutside(event: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setOpen(false);
        setView("main");
        setBanReason("");
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [open]);

  // Close on Escape
  useEffect(() => {
    if (!open) return;
    function handleEscape(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setOpen(false);
        setView("main");
        setBanReason("");
      }
    }
    document.addEventListener("keydown", handleEscape);
    return () => document.removeEventListener("keydown", handleEscape);
  }, [open]);

  if (!hasAnyPermission) return null;

  function handleToggle() {
    setOpen((v) => !v);
    setView("main");
    setBanReason("");
  }

  async function handleKickConfirm() {
    if (pending) return;
    setPending(true);
    try {
      await onKick(member.pubkey);
      setOpen(false);
      setView("main");
    } finally {
      setPending(false);
    }
  }

  async function handleBanConfirm() {
    if (pending) return;
    setPending(true);
    try {
      await onBan(member.pubkey, banReason || undefined);
      setOpen(false);
      setView("main");
      setBanReason("");
    } finally {
      setPending(false);
    }
  }

  async function handleRoleToggle(roleId: string) {
    if (pendingRoleId) return;
    const hasRole = memberRoleIds.includes(roleId);
    setPendingRoleId(roleId);
    try {
      if (hasRole) {
        await onRemoveRole(member.pubkey, roleId);
      } else {
        await onAssignRole(member.pubkey, roleId);
      }
    } finally {
      setPendingRoleId(null);
    }
  }

  return (
    <div ref={menuRef} className="relative">
      <button
        onClick={handleToggle}
        className="rounded-md px-1.5 py-0.5 text-ctp-subtext0 hover:bg-ctp-surface0 hover:text-ctp-text"
        aria-label="Member actions"
      >
        ⋯
      </button>

      {open && (
        <div className="absolute right-0 top-full z-50 mt-1 w-48 rounded-lg border border-ctp-surface1 bg-ctp-surface0 py-1 shadow-lg">
          {view === "main" && (
            <>
              {canKick && (
                <button
                  onClick={() => setView("kick-confirm")}
                  className="flex w-full items-center gap-2 px-3 py-1.5 text-left text-sm text-ctp-yellow hover:bg-ctp-surface1"
                >
                  Kick
                </button>
              )}
              {canBan && (
                <button
                  onClick={() => setView("ban-confirm")}
                  className="flex w-full items-center gap-2 px-3 py-1.5 text-left text-sm text-ctp-red hover:bg-ctp-surface1"
                >
                  Ban
                </button>
              )}
              {canManageRoles && assignableRoles.length > 0 && (
                <>
                  {(canKick || canBan) && (
                    <div className="mx-2 my-1 border-t border-ctp-overlay0" />
                  )}
                  <div className="px-3 py-1 text-xs font-semibold text-ctp-subtext0">
                    Roles
                  </div>
                  {assignableRoles.map((role) => {
                    const hasRole = memberRoleIds.includes(role.id);
                    const isLoading = pendingRoleId === role.id;
                    return (
                      <button
                        key={role.id}
                        onClick={() => void handleRoleToggle(role.id)}
                        disabled={isLoading}
                        className="flex w-full items-center gap-2 px-3 py-1.5 text-left text-sm text-ctp-text hover:bg-ctp-surface1 disabled:opacity-60"
                      >
                        <span className="w-4 text-center text-xs">
                          {isLoading ? "…" : hasRole ? "✓" : ""}
                        </span>
                        <span className="truncate">{role.name}</span>
                      </button>
                    );
                  })}
                </>
              )}
            </>
          )}

          {view === "kick-confirm" && (
            <div className="px-3 py-2">
              <p className="mb-2 text-xs text-ctp-subtext1">
                Kick this member?
              </p>
              <div className="flex gap-2">
                <button
                  onClick={() => void handleKickConfirm()}
                  disabled={pending}
                  className="flex-1 rounded-md bg-ctp-yellow/20 px-2 py-1 text-xs text-ctp-yellow hover:bg-ctp-yellow/30 disabled:opacity-60"
                >
                  {pending ? "…" : "Confirm"}
                </button>
                <button
                  onClick={() => setView("main")}
                  disabled={pending}
                  className="flex-1 rounded-md bg-ctp-surface1 px-2 py-1 text-xs text-ctp-subtext1 hover:bg-ctp-overlay0"
                >
                  Cancel
                </button>
              </div>
            </div>
          )}

          {view === "ban-confirm" && (
            <div className="px-3 py-2">
              <p className="mb-2 text-xs text-ctp-subtext1">
                Ban this member?
              </p>
              <input
                type="text"
                placeholder="Reason (optional)"
                value={banReason}
                onChange={(e) => setBanReason(e.target.value)}
                className="mb-2 w-full rounded-md border border-ctp-surface1 bg-ctp-base px-2 py-1 text-xs text-ctp-text placeholder:text-ctp-overlay1 focus:outline-none focus:border-ctp-blue"
              />
              <div className="flex gap-2">
                <button
                  onClick={() => void handleBanConfirm()}
                  disabled={pending}
                  className="flex-1 rounded-md bg-ctp-red/20 px-2 py-1 text-xs text-ctp-red hover:bg-ctp-red/30 disabled:opacity-60"
                >
                  {pending ? "…" : "Confirm"}
                </button>
                <button
                  onClick={() => {
                    setView("main");
                    setBanReason("");
                  }}
                  disabled={pending}
                  className="flex-1 rounded-md bg-ctp-surface1 px-2 py-1 text-xs text-ctp-subtext1 hover:bg-ctp-overlay0"
                >
                  Cancel
                </button>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
