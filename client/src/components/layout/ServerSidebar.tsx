import { useState } from "react";

import { ThemeSwitcher } from "../settings/ThemeSwitcher";
import { useAppStore } from "../../stores/appStore";

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
  const [themeOpen, setThemeOpen] = useState(false);

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
    </aside>
  );
}
