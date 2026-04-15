import { create } from "zustand";
import { persist } from "zustand/middleware";

import { applyTheme, defaultTheme } from "../theme/apply";
import type { ThemeName } from "../theme/types";

export interface ServerConnection {
  id: string;
  address: string;
  name: string;
}

interface AppStore {
  currentServerId: string | null;
  servers: Record<string, ServerConnection>;
  theme: ThemeName;
  addServer: (address: string, name: string) => string;
  removeServer: (id: string) => void;
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
      addServer: (address, name) => {
        const normalized = normalizeAddress(address);
        const id = normalized;
        const existing = get().servers[id];
        if (existing) {
          set((state) => ({
            servers: {
              ...state.servers,
              [id]: { ...existing, name },
            },
            currentServerId: id,
          }));
          return id;
        }

        set((state) => ({
          servers: {
            ...state.servers,
            [id]: { id, address: normalized, name },
          },
          currentServerId: id,
        }));

        return id;
      },
      removeServer: (id) => {
        set((state) => {
          const nextServers = { ...state.servers };
          delete nextServers[id];
          return {
            servers: nextServers,
            currentServerId: state.currentServerId === id ? null : state.currentServerId,
          };
        });
      },
      setCurrentServer: (id) => set({ currentServerId: id }),
      setTheme: (theme) => {
        applyTheme(theme);
        set({ theme });
      },
      initTheme: () => {
        const theme = get().theme;
        applyTheme(theme);
      },
    }),
    {
      name: "decentcom-app-storage",
    },
  ),
);
