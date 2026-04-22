import { useEffect } from "react";

import { CreateInviteDialog } from "../invites/CreateInviteDialog";
import { AllowlistEditor } from "./AllowlistEditor";
import { BanList } from "./BanList";
import { BotApprovalPanel } from "./BotApprovalPanel";
import { MembershipSettings } from "./MembershipSettings";
import { InviteList } from "../invites/InviteList";
import { Modal } from "../ui/Modal";
import {
  BAN_MEMBERS,
  MANAGE_INVITES,
  MANAGE_ROLES,
  MANAGE_SERVER,
  usePermissions,
} from "../../hooks/usePermissions";
import { useInvitesStore } from "../../stores/invites";
import { useMembersStore } from "../../stores/members";
import { useServerStore } from "../../stores/serverStore";

interface ServerSettingsModalProps {
  onClose: () => void;
}

export function ServerSettingsModal({ onClose }: ServerSettingsModalProps) {
  const address = useServerStore((state) => state.address);
  const membershipMode = useServerStore((state) => state.membershipMode);
  const token = useServerStore((state) => state.sessionToken);
  const roles = useServerStore((state) => state.roles);

  const permissions = usePermissions();
  const canManageInvites = permissions.has(MANAGE_INVITES);
  const canBanMembers = permissions.has(BAN_MEMBERS);
  const canManageServer = permissions.has(MANAGE_SERVER);
  const canManageRoles = permissions.has(MANAGE_ROLES);

  const invites = useInvitesStore((state) => state.invites);
  const loadingInvites = useInvitesStore((state) => state.loading);
  const fetchInvites = useInvitesStore((state) => state.listInvites);
  const createInvite = useInvitesStore((state) => state.createInvite);
  const revokeInvite = useInvitesStore((state) => state.revokeInvite);

  const bans = useMembersStore((state) => state.bans);
  const allowlist = useMembersStore((state) => state.allowlist);
  const pendingBots = useMembersStore((state) => state.pendingBots);
  const fetchBans = useMembersStore((state) => state.fetchBans);
  const fetchAllowlist = useMembersStore((state) => state.fetchAllowlist);
  const fetchPendingBots = useMembersStore((state) => state.fetchPendingBots);
  const unbanMember = useMembersStore((state) => state.unban);
  const addAllowlist = useMembersStore((state) => state.addAllowlistEntry);
  const removeAllowlist = useMembersStore((state) => state.removeAllowlistEntry);
  const approveBotEntry = useMembersStore((state) => state.approveBotEntry);
  const revokeBotEntry = useMembersStore((state) => state.revokeBotEntry);

  const hasAnyPermission = canManageInvites || canBanMembers || canManageServer || canManageRoles;

  useEffect(() => {
    if (!address || !token) return;
    if (canManageInvites) void fetchInvites(address, token);
    if (canBanMembers) void fetchBans(address, token);
    if (canManageServer) {
      void fetchAllowlist(address, token);
      void fetchPendingBots(address, token);
    }
  }, [
    address,
    token,
    canManageInvites,
    canBanMembers,
    canManageServer,
    fetchInvites,
    fetchBans,
    fetchAllowlist,
    fetchPendingBots,
  ]);

  return (
    <Modal onClose={onClose}>
      <div className="w-[32rem] p-6">
        <div className="mb-4 flex items-center justify-between">
          <h2 className="text-lg font-bold text-ctp-text">Server Settings</h2>
          <button
            onClick={onClose}
            className="text-ctp-subtext1 hover:text-ctp-text transition"
            aria-label="Close"
          >
            ✕
          </button>
        </div>

        {!address || !token ? (
          <p className="text-ctp-subtext1 text-sm">Connecting to server…</p>
        ) : !hasAnyPermission ? (
          <p className="text-ctp-subtext1 text-sm">You don't have permission to manage this server.</p>
        ) : (
          <div className="grid gap-4">
            <MembershipSettings mode={membershipMode} />
            {canManageInvites && (
              <>
                <CreateInviteDialog
                  roles={roles}
                  onCreate={async (input) => {
                    if (!address || !token) throw new Error("Not connected");
                    const created = await createInvite(address, token, input);
                    await fetchInvites(address, token);
                    return created;
                  }}
                />
                <InviteList
                  invites={invites}
                  loading={loadingInvites}
                  onRevoke={async (code) => {
                    if (!address || !token) throw new Error("Not connected");
                    await revokeInvite(address, token, code);
                    await fetchInvites(address, token);
                  }}
                />
              </>
            )}
            {canBanMembers && (
              <BanList
                bans={bans}
                onUnban={async (pubkey) => {
                  if (!address || !token) throw new Error("Not connected");
                  await unbanMember(address, token, pubkey);
                }}
              />
            )}
            {canManageServer && (
              <>
                <AllowlistEditor
                  entries={allowlist}
                  onAdd={async (pubkey) => {
                    if (!address || !token) throw new Error("Not connected");
                    await addAllowlist(address, token, pubkey);
                  }}
                  onRemove={async (pubkey) => {
                    if (!address || !token) throw new Error("Not connected");
                    await removeAllowlist(address, token, pubkey);
                  }}
                />
                <BotApprovalPanel
                  pendingBots={pendingBots}
                  onApprove={async (pubkey) => {
                    if (!address || !token) throw new Error("Not connected");
                    await approveBotEntry(address, token, pubkey);
                  }}
                  onRevoke={async (pubkey) => {
                    if (!address || !token) throw new Error("Not connected");
                    await revokeBotEntry(address, token, pubkey);
                    await fetchPendingBots(address, token);
                  }}
                />
              </>
            )}
          </div>
        )}
      </div>
    </Modal>
  );
}
