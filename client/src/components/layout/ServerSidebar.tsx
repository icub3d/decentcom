import { useEffect, useRef, useState } from "react";

import { CreateInviteDialog } from "../invites/CreateInviteDialog";
import { MemberList } from "../members/MemberList";
import { AllowlistEditor } from "../settings/AllowlistEditor";
import { BanList } from "../settings/BanList";
import { MembershipSettings } from "../settings/MembershipSettings";
import { InviteList } from "../invites/InviteList";
import { ProfileEditor } from "../profile/ProfileEditor";
import { ThemeSwitcher } from "../settings/ThemeSwitcher";
import { BAN_MEMBERS, MANAGE_INVITES, MANAGE_SERVER, usePermissions } from "../../hooks/usePermissions";
import { useAppStore } from "../../stores/appStore";
import { useInvitesStore } from "../../stores/invites";
import { useMembersStore } from "../../stores/members";
import { useServerStore } from "../../stores/serverStore";

interface ServerSidebarProps {
  servers: Array<{ id: string; address: string }>;
  currentServerId: string | null;
  onSelectServer: (id: string) => void;
}

function initials(address: string): string {
  return new URL(address).hostname.slice(0, 2).toUpperCase();
}

export function ServerSidebar({ servers, currentServerId, onSelectServer }: ServerSidebarProps) {
  const { theme, setTheme, setCurrentServer } = useAppStore();
  const address = useServerStore((state) => state.address);
  const token = useServerStore((state) => state.sessionToken);
  const roles = useServerStore((state) => state.roles);
  const permissions = usePermissions();
  const canManageInvites = permissions.has(MANAGE_INVITES);
  const canBanMembers = permissions.has(BAN_MEMBERS);
  const canManageServer = permissions.has(MANAGE_SERVER);

  const invites = useInvitesStore((state) => state.invites);
  const loadingInvites = useInvitesStore((state) => state.loading);
  const fetchInvites = useInvitesStore((state) => state.listInvites);
  const createInvite = useInvitesStore((state) => state.createInvite);
  const revokeInvite = useInvitesStore((state) => state.revokeInvite);

  const members = useMembersStore((state) => state.members);
  const bans = useMembersStore((state) => state.bans);
  const allowlist = useMembersStore((state) => state.allowlist);
  const fetchMembers = useMembersStore((state) => state.fetchMembers);
  const fetchBans = useMembersStore((state) => state.fetchBans);
  const fetchAllowlist = useMembersStore((state) => state.fetchAllowlist);
  const kickMember = useMembersStore((state) => state.kick);
  const banMember = useMembersStore((state) => state.ban);
  const unbanMember = useMembersStore((state) => state.unban);
  const addAllowlist = useMembersStore((state) => state.addAllowlistEntry);
  const removeAllowlist = useMembersStore((state) => state.removeAllowlistEntry);

  const [themeOpen, setThemeOpen] = useState(false);
  const [inviteOpen, setInviteOpen] = useState(false);
  const [membersOpen, setMembersOpen] = useState(false);
  const [profileOpen, setProfileOpen] = useState(false);

  const sidebarRef = useRef<HTMLElement>(null);

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (sidebarRef.current && !sidebarRef.current.contains(event.target as Node)) {
        setThemeOpen(false);
        setInviteOpen(false);
        setMembersOpen(false);
        setProfileOpen(false);
      }
    }

    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, []);

  async function refreshInvites() {
    if (!address || !token) {
      return;
    }
    await fetchInvites(address, token);
  }

  async function refreshMembersPanel() {
    if (!address || !token) {
      return;
    }
    await fetchMembers(address, token);
    if (canBanMembers) {
      await fetchBans(address, token);
    }
    if (canManageServer) {
      await fetchAllowlist(address, token);
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
          <button
            key={server.id}
            onClick={() => onSelectServer(server.id)}
            title={server.address}
            className={`h-12 w-12 rounded-xl font-black text-sm transition ${
              active
                ? "bg-ctp-blue text-ctp-crust"
                : "bg-ctp-surface0 text-ctp-subtext1 hover:bg-ctp-surface1"
            }`}
          >
            {initials(server.address)}
          </button>
        );
      })}

      <div className="mt-auto">
        {canManageInvites && (
          <button
            onClick={() => {
              const next = !inviteOpen;
              setInviteOpen(next);
              if (next) {
                void refreshInvites();
              }
            }}
            title="Invite settings"
            className="mb-3 h-12 w-12 rounded-xl bg-ctp-surface0 text-ctp-subtext1 hover:bg-ctp-surface1 transition flex items-center justify-center text-[10px] font-bold"
          >
            INV
          </button>
        )}
        <button
          onClick={() => {
            const next = !membersOpen;
            setMembersOpen(next);
            if (next) {
              void refreshMembersPanel();
            }
          }}
          title="Membership"
          className="mb-3 h-12 w-12 rounded-xl bg-ctp-surface0 text-ctp-subtext1 hover:bg-ctp-surface1 transition"
        >
          👥
        </button>
        <button
          onClick={() => setProfileOpen((v) => !v)}
          title="My profile"
          className="mb-3 h-12 w-12 rounded-xl bg-ctp-surface0 text-ctp-subtext1 hover:bg-ctp-surface1 transition"
        >
          👤
        </button>
        <button
          onClick={() => setThemeOpen((v) => !v)}
          title="Theme settings"
          className="h-12 w-12 rounded-xl bg-ctp-surface0 text-ctp-subtext1 hover:bg-ctp-surface1 transition"
        >
          ⚙
        </button>
      </div>

      {themeOpen && (
        <div className="absolute bottom-3 left-20 z-10 w-56 pl-2">
          <ThemeSwitcher
            theme={theme}
            onThemeSelect={(next) => {
              setTheme(next);
            }}
          />
        </div>
      )}

      {profileOpen && (
        <div className="absolute bottom-3 left-20 z-10 w-80 pl-2">
          <ProfileEditor />
        </div>
      )}

      {inviteOpen && canManageInvites && (
        <div className="absolute bottom-3 left-20 z-10 w-104 pl-2">
          <div className="grid gap-3">
            <CreateInviteDialog
              roles={roles}
              onCreate={async (input) => {
                if (!address || !token) {
                  throw new Error("Not connected");
                }
                const created = await createInvite(address, token, input);
                await refreshInvites();
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
                await refreshInvites();
              }}
            />
          </div>
        </div>
      )}

      {membersOpen && (
        <div className="absolute bottom-3 left-20 z-10 w-lg pl-2">
          <div className="grid gap-3">
            <MembershipSettings mode="server-config" />
            <MemberList
              members={members}
              onKick={async (pubkey) => {
                if (!address || !token) {
                  throw new Error("Not connected");
                }
                await kickMember(address, token, pubkey);
              }}
              onBan={async (pubkey, reason) => {
                if (!address || !token) {
                  throw new Error("Not connected");
                }
                await banMember(address, token, pubkey, reason);
              }}
            />
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
            )}
          </div>
        </div>
      )}
    </aside>
  );
}
