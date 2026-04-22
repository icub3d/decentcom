import { useEffect, useRef, useState } from "react";

import { ServerSettingsModal } from "../settings/ServerSettingsModal";
import { ProfileEditor } from "../profile/ProfileEditor";
import { ThemeSwitcher } from "../settings/ThemeSwitcher";
import { AccountSwitcher } from "../accounts/AccountSwitcher";
import { useAppStore } from "../../stores/appStore";

interface ServerSidebarProps {
  servers: Array<{ id: string; address: string; name?: string }>;
  currentServerId: string | null;
  onSelectServer: (id: string) => void;
  onSwitchAccount: (pubkey: string) => void | Promise<void>;
}

type OpenPanel = null | "user-settings";

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

  const [openPanel, setOpenPanel] = useState<OpenPanel>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);

  const sidebarRef = useRef<HTMLElement>(null);

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      const target = event.target as HTMLElement;
      if (target.closest("[data-modal-backdrop]")) return;
      if (sidebarRef.current && !sidebarRef.current.contains(target)) {
        setOpenPanel(null);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  function togglePanel(panel: OpenPanel) {
    setOpenPanel((prev) => (prev === panel ? null : panel));
  }

  function handleOpenSettings(serverId: string) {
    if (serverId !== currentServerId) {
      onSelectServer(serverId);
    }
    setSettingsOpen(true);
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
            <button
              onClick={(e) => {
                e.stopPropagation();
                handleOpenSettings(server.id);
              }}
              className="absolute -right-1 -bottom-1 hidden h-5 w-5 items-center justify-center rounded-full bg-ctp-surface1 text-[10px] text-ctp-subtext1 shadow-lg hover:bg-ctp-blue hover:text-ctp-crust hover:scale-110 group-hover:flex"
              title="Server settings"
            >
              ⚙
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

      {settingsOpen && (
        <ServerSettingsModal onClose={() => setSettingsOpen(false)} />
      )}
    </aside>
  );
}
