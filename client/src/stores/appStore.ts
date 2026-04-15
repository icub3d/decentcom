import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";

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

/** Get the storage key namespaced by public key, or a default key. */
function getStorageKey(): string {
  const pubkey = localStorage.getItem("decentcom-active-pubkey");
  if (pubkey) {
    return `decentcom-app-storage-${pubkey}`;
  }
  return "decentcom-app-storage";
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
      name: getStorageKey(),
      storage: createJSONStorage(() => localStorage),
    },
  ),
);

/**
 * Re-initialize the app store with a new storage key for the given pubkey.
 * Call this when switching accounts to load that account's persisted state.
 */
export function switchAppStoreAccount(pubkey: string | null) {
  localStorage.setItem("decentcom-active-pubkey", pubkey ?? "");
  // Rehydrate from the new storage key.
  const key = pubkey ? `decentcom-app-storage-${pubkey}` : "decentcom-app-storage";
  const raw = localStorage.getItem(key);
  if (raw) {
    try {
      const parsed = JSON.parse(raw);
      if (parsed?.state) {
        useAppStore.setState(parsed.state);
      }
    } catch {
      // corrupt storage — start fresh
    }
  } else {
    // No persisted state for this account — reset to defaults.
    useAppStore.setState({
      currentServerId: null,
      servers: {},
      theme: defaultTheme(),
    });
  }
  // Update the persist name so future writes go to the right key.
  useAppStore.persist.setOptions({ name: key });
}
