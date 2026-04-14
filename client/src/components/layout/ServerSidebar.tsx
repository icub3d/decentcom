import { useState } from "react";

import { CreateInviteDialog } from "../invites/CreateInviteDialog";
import { InviteList } from "../invites/InviteList";
import { ThemeSwitcher } from "../settings/ThemeSwitcher";
import { MANAGE_INVITES, usePermissions } from "../../hooks/usePermissions";
import { useAppStore } from "../../stores/appStore";
import { useInvitesStore } from "../../stores/invites";
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
  const { theme, setTheme } = useAppStore();
  const address = useServerStore((state) => state.address);
  const token = useServerStore((state) => state.sessionToken);
  const roles = useServerStore((state) => state.roles);
  const permissions = usePermissions();
  const canManageInvites = permissions.has(MANAGE_INVITES);

  const invites = useInvitesStore((state) => state.invites);
  const loadingInvites = useInvitesStore((state) => state.loading);
  const fetchInvites = useInvitesStore((state) => state.listInvites);
  const createInvite = useInvitesStore((state) => state.createInvite);
  const revokeInvite = useInvitesStore((state) => state.revokeInvite);

  const [themeOpen, setThemeOpen] = useState(false);
  const [inviteOpen, setInviteOpen] = useState(false);

  async function refreshInvites() {
    if (!address || !token) {
      return;
    }
    await fetchInvites(address, token);
  }

  return (
    <aside className="w-20 border-r border-ctp-overlay0 bg-ctp-crust p-3 flex flex-col gap-3 relative">
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
            className="mb-3 h-12 w-12 rounded-xl bg-ctp-surface0 text-ctp-subtext1 hover:bg-ctp-surface1 transition"
          >
            +
          </button>
        )}
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

      {inviteOpen && canManageInvites && (
        <div className="absolute bottom-3 left-20 z-10 w-[26rem] pl-2">
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
    </aside>
  );
}
