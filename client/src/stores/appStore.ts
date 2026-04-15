import { create } from "zustand";
import { persist } from "zustand/middleware";

import { applyTheme, defaultTheme, loadTheme, saveTheme } from "../theme/apply";
import type { ThemeName } from "../theme/types";

export interface ServerConnection {
  id: string;
  address: string;
}

interface AppStore {
  currentServerId: string | null;
  servers: Record<string, ServerConnection>;
  theme: ThemeName;
  addServer: (address: string) => string;
  setCurrentServer: (id: string | null) => void;
  setTheme: (theme: ThemeName) => void;
  initTheme: () => void;
}

function normalizeAddress(address: string): string {
  const trimmed = address.trim().replace(/\/$/, "");
  if (trimmed.startsWith("http://") || trimmed.startsWith("https://")) {
    return trimmed;
  }
  return `http://${trimmed}`;
}

export const useAppStore = create<AppStore>()(
  persist(
    (set, get) => ({
      currentServerId: null,
      servers: {},
      theme: defaultTheme(),
      addServer: (address) => {
        const normalized = normalizeAddress(address);
        const id = normalized;
        const existing = get().servers[id];
        if (existing) {
          set({ currentServerId: id });
          return id;
        }

        set((state) => ({
          servers: {
            ...state.servers,
            [id]: { id, address: normalized },
          },
          currentServerId: id,
        }));

        return id;
      },
      setCurrentServer: (id) => set({ currentServerId: id }),
      setTheme: (theme) => {
        applyTheme(theme);
        saveTheme(theme);
        set({ theme });
      },
      initTheme: () => {
        const theme = loadTheme();
        applyTheme(theme);
        set({ theme });
      },
    }),
    {
      name: "decentcom-app-storage",
    },
  ),
);
