import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";

import { applyTheme, defaultTheme } from "../theme/apply";
import type { ThemeName } from "../theme/types";

export interface ServerConnection {
  id: string;
  address: string;
  name: string;
}

export interface AppBackupState {
  version: 1;
  servers: Record<string, ServerConnection>;
  theme: ThemeName;
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
  getStateForBackup: () => AppBackupState;
  loadFromBackup: (state: AppBackupState) => void;
}

function normalizeAddress(address: string): string {
  const trimmed = address.trim().replace(/\/$/, "");
  if (trimmed.startsWith("http://") || trimmed.startsWith("https://")) {
    return trimmed;
  }
  return `http://${trimmed}`;
}

/** Storage key for a given pubkey, or the default un-namespaced key. */
function storageKeyFor(pubkey: string | null): string {
  return pubkey ? `decentcom-app-storage-${pubkey}` : "decentcom-app-storage";
}

/** Read the active-pubkey marker from localStorage (may be null on first run). */
function getActivePubkeyFromStorage(): string | null {
  return localStorage.getItem("decentcom-active-pubkey") || null;
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
      getStateForBackup: () => {
        const { servers, theme } = get();
        return { version: 1, servers, theme };
      },
      loadFromBackup: (state) => {
        applyTheme(state.theme);
        set({ servers: state.servers, theme: state.theme, currentServerId: null });
      },
    }),
    {
      name: storageKeyFor(getActivePubkeyFromStorage()),
      storage: createJSONStorage(() => localStorage),
    },
  ),
);

/**
 * Called once after identity loads to ensure the persist middleware writes
 * to the correct per-account key from the very first session.
 */
export async function initAppStoreForAccount(pubkey: string) {
  const current = getActivePubkeyFromStorage();
  if (current === pubkey) return; // already namespaced

  const oldKey = storageKeyFor(current);
  const newKey = storageKeyFor(pubkey);

  // Only migrate data from the un-namespaced default key (first-ever run).
  // Never copy one account's servers/settings into another account.
  if (current === null) {
    const existing = localStorage.getItem(oldKey);
    if (existing && !localStorage.getItem(newKey)) {
      localStorage.setItem(newKey, existing);
    }
  }

  localStorage.setItem("decentcom-active-pubkey", pubkey);

  // Point persist at the new key, then rehydrate so the store picks it up.
  useAppStore.persist.setOptions({ name: newKey });

  // Reset to defaults BEFORE rehydrate ONLY if we are switching between accounts.
  // This avoids clearing the store on the first run of a session if we are
  // just namespacing the initial default state.
  if (current !== null) {
    useAppStore.setState({
      currentServerId: null,
      servers: {},
      theme: defaultTheme(),
    });
  }

  await useAppStore.persist.rehydrate();
}

/**
 * Switch the app store to a different account's persisted state.
 * Must be called AFTER the backend active account has been changed.
 */
export function switchAppStoreAccount(oldPubkey: string | null, newPubkey: string) {
  // Guard: nothing to do if we're already on the target account.
  if (oldPubkey === newPubkey) return;

  // 1. Snapshot current state into the old account's storage key.
  if (oldPubkey) {
    const { currentServerId, servers, theme } = useAppStore.getState();
    const blob = JSON.stringify({ state: { currentServerId, servers, theme }, version: 0 });
    localStorage.setItem(storageKeyFor(oldPubkey), blob);
  }

  // 2. Update the active-pubkey marker.
  localStorage.setItem("decentcom-active-pubkey", newPubkey);

  // 3. Point the persist middleware at the new key BEFORE setting state,
  //    so the auto-save writes to the correct location.
  const newKey = storageKeyFor(newPubkey);
  useAppStore.persist.setOptions({ name: newKey });

  // 4. Load the new account's state (or reset to defaults).
  const raw = localStorage.getItem(newKey);
  if (raw) {
    try {
      const parsed = JSON.parse(raw);
      if (parsed?.state) {
        useAppStore.setState(parsed.state);
        return;
      }
    } catch {
      // corrupt — fall through to reset
    }
  }
  useAppStore.setState({
    currentServerId: null,
    servers: {},
    theme: defaultTheme(),
  });
}
