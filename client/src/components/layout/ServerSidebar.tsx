import { useEffect, useRef, useState } from "react";

import { CreateInviteDialog } from "../invites/CreateInviteDialog";
import { AllowlistEditor } from "../settings/AllowlistEditor";
import { BanList } from "../settings/BanList";
import { BotApprovalPanel } from "../settings/BotApprovalPanel";
import { MembershipSettings } from "../settings/MembershipSettings";
import { InviteList } from "../invites/InviteList";
import { ProfileEditor } from "../profile/ProfileEditor";
import { ThemeSwitcher } from "../settings/ThemeSwitcher";
import { AccountSwitcher } from "../accounts/AccountSwitcher";
import {
  BAN_MEMBERS,
  MANAGE_INVITES,
  MANAGE_ROLES,
  MANAGE_SERVER,
  usePermissions,
} from "../../hooks/usePermissions";
import { useAppStore } from "../../stores/appStore";
import { useInvitesStore } from "../../stores/invites";
import { useMembersStore } from "../../stores/members";
import { useServerStore } from "../../stores/serverStore";

interface ServerSidebarProps {
  servers: Array<{ id: string; address: string; name?: string }>;
  currentServerId: string | null;
  onSelectServer: (id: string) => void;
  onSwitchAccount: (pubkey: string) => void | Promise<void>;
}

type OpenPanel = null | "user-settings" | "server-settings";

function initials(name: string | undefined, address: string): string {
  if (!name || name.trim() === "") {
    try {
      return new URL(address).hostname.slice(0, 2).toUpperCase();
    } catch {
      return "??";
    }
  }
  const parts = name.split(/\s+/).filter(Boolean);
  if (parts.length >= 2) {
    return parts
      .map((p) => p[0])
      .join("")
      .slice(0, 3)
      .toUpperCase();
  }
  return name.slice(0, 2).toUpperCase();
}

export function ServerSidebar({ servers, currentServerId, onSelectServer, onSwitchAccount }: ServerSidebarProps) {
  const { theme, setTheme, setCurrentServer, removeServer } = useAppStore();
  const address = useServerStore((state) => state.address);

  const token = useServerStore((state) => state.sessionToken);
  const roles = useServerStore((state) => state.roles);
  const permissions = usePermissions();
  const canManageInvites = permissions.has(MANAGE_INVITES);
  const canBanMembers = permissions.has(BAN_MEMBERS);
  const canManageServer = permissions.has(MANAGE_SERVER);
  const canManageRoles = permissions.has(MANAGE_ROLES);
  const showServerSettings = canManageInvites || canBanMembers || canManageServer || canManageRoles;

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

  const [openPanel, setOpenPanel] = useState<OpenPanel>(null);

  const sidebarRef = useRef<HTMLElement>(null);

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      const target = event.target as HTMLElement;
      // Ignore clicks inside portaled modals so they don't close the sidebar panel
      if (target.closest("[data-modal-backdrop]")) return;
      if (sidebarRef.current && !sidebarRef.current.contains(target)) {
        setOpenPanel(null);
      }
    }

    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, []);

  function togglePanel(panel: OpenPanel) {
    setOpenPanel((prev) => (prev === panel ? null : panel));
  }

  async function refreshServerSettings() {
    if (!address || !token) {
      return;
    }
    if (canManageInvites) {
      await fetchInvites(address, token);
    }
    if (canBanMembers) {
      await fetchBans(address, token);
    }
    if (canManageServer) {
      await fetchAllowlist(address, token);
      await fetchPendingBots(address, token);
    }
  }

  return (
    <aside ref={sidebarRef} className="w-20 border-r border-ctp-overlay0 bg-ctp-crust p-3 flex flex-col gap-3 relative">
      <button
        onClick={() => setCurrentServer(null)}
        title="Add a new server"
        className="h-12 w-12 rounded-full border-2 border-dashed border-ctp-overlay0 text-ctp-subtext1 hover:bg-ctp-surface0 hover:border-ctp-blue hover:text-ctp-blue transition flex items-center justify-center font-bold text-xl"
      >
        +
      </button>

      {servers.map((server) => {
        const active = server.id === currentServerId;
        return (
          <div key={server.id} className="group relative">
            <button
              onClick={() => onSelectServer(server.id)}
              title={server.name || server.address}
              className={`h-12 w-12 rounded-xl font-black text-sm transition ${
                active
                  ? "bg-ctp-blue text-ctp-crust"
                  : "bg-ctp-surface0 text-ctp-subtext1 hover:bg-ctp-surface1"
              }`}
            >
              {initials(server.name, server.address)}
            </button>
            <button
              onClick={(e) => {
                e.stopPropagation();
                if (confirm(`Remove connection to ${server.address}?`)) {
                  removeServer(server.id);
                }
              }}
              className="absolute -right-1 -top-1 hidden h-5 w-5 items-center justify-center rounded-full bg-ctp-red text-[10px] font-bold text-ctp-crust shadow-lg hover:scale-110 group-hover:flex"
              title="Remove server"
            >
              ×
            </button>
          </div>
        );
      })}

      <div className="mt-auto flex flex-col gap-3">
        <button
          onClick={() => togglePanel("user-settings")}
          title="User settings"
          className={`h-12 w-12 rounded-xl transition flex items-center justify-center ${
            openPanel === "user-settings"
              ? "bg-ctp-blue text-ctp-crust"
              : "bg-ctp-surface0 text-ctp-subtext1 hover:bg-ctp-surface1"
          }`}
        >
          👤
        </button>
        {showServerSettings && (
          <button
            onClick={() => {
              const willOpen = openPanel !== "server-settings";
              togglePanel("server-settings");
              if (willOpen) {
                void refreshServerSettings();
              }
            }}
            title="Server settings"
            className={`h-12 w-12 rounded-xl transition flex items-center justify-center ${
              openPanel === "server-settings"
                ? "bg-ctp-blue text-ctp-crust"
                : "bg-ctp-surface0 text-ctp-subtext1 hover:bg-ctp-surface1"
            }`}
          >
            ⚙
          </button>
        )}
      </div>

      {openPanel === "user-settings" && (
        <div className="absolute bottom-3 left-20 z-10 w-80 pl-2 max-h-[80vh] overflow-y-auto">
          <div className="grid gap-3">
            <AccountSwitcher
              onSwitchAccount={onSwitchAccount}
            />
            <ProfileEditor />
            <ThemeSwitcher
              theme={theme}
              onThemeSelect={(next) => {
                setTheme(next);
              }}
            />
          </div>
        </div>
      )}

      {openPanel === "server-settings" && showServerSettings && (
        <div className="absolute bottom-3 left-20 z-10 w-104 pl-2">
          <div className="grid gap-3">
            <MembershipSettings mode="server-config" />
            {canManageInvites && (
              <>
                <CreateInviteDialog
                  roles={roles}
                  onCreate={async (input) => {
                    if (!address || !token) {
                      throw new Error("Not connected");
                    }
                    const created = await createInvite(address, token, input);
                    await fetchInvites(address, token);
                    return created;
                  }}
                />
                <InviteList
                  invites={invites}
                  loading={loadingInvites}
                  onRevoke={async (code) => {
                    if (!address || !token) {
                      throw new Error("Not connected");
                    }
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
                  if (!address || !token) {
                    throw new Error("Not connected");
                  }
                  await unbanMember(address, token, pubkey);
                }}
              />
            )}
            {canManageServer && (
              <>
                <AllowlistEditor
                  entries={allowlist}
                  onAdd={async (pubkey) => {
                    if (!address || !token) {
                      throw new Error("Not connected");
                    }
                    await addAllowlist(address, token, pubkey);
                  }}
                  onRemove={async (pubkey) => {
                    if (!address || !token) {
                      throw new Error("Not connected");
                    }
                    await removeAllowlist(address, token, pubkey);
                  }}
                />
                <BotApprovalPanel
                  pendingBots={pendingBots}
                  onApprove={async (pubkey) => {
                    if (!address || !token) {
                      throw new Error("Not connected");
                    }
                    await approveBotEntry(address, token, pubkey);
                  }}
                  onRevoke={async (pubkey) => {
                    if (!address || !token) {
                      throw new Error("Not connected");
                    }
                    await revokeBotEntry(address, token, pubkey);
                    await fetchPendingBots(address, token);
                  }}
                />
              </>
            )}
          </div>
        </div>
      )}
    </aside>
  );
}
