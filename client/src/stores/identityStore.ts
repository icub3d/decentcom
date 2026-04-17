import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface AccountInfo {
  pubkey: string;
  label: string | null;
  active: boolean;
}

interface IdentityStore {
  accounts: AccountInfo[];
  activeAccount: string | null;
  loading: boolean;
  error: string | null;

  /** Refresh accounts list from Tauri backend. */
  refresh: () => Promise<void>;

  /** Switch the active account (disconnects current session). */
  setActiveAccount: (pubkey: string) => Promise<void>;

  /** Delete an account from the keychain. */
  deleteAccount: (pubkey: string) => Promise<void>;

  /** Rename an account label. */
  renameAccount: (pubkey: string, label: string) => Promise<void>;
}

export const useIdentityStore = create<IdentityStore>()((set) => ({
  accounts: [],
  activeAccount: null,
  loading: false,
  error: null,

  refresh: async () => {
    try {
      set({ loading: true, error: null });
      const accounts = await invoke<AccountInfo[]>("list_accounts");
      const active = accounts.find((a) => a.active)?.pubkey ?? null;
      set({ accounts, activeAccount: active, loading: false });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
        loading: false,
      });
    }
  },

  setActiveAccount: async (pubkey: string) => {
    try {
      set({ loading: true, error: null });
      await invoke("set_active_account", { pubkey });
      const accounts = await invoke<AccountInfo[]>("list_accounts");
      const active = accounts.find((a) => a.active)?.pubkey ?? null;
      if (active !== pubkey) {
        const msg = `Backend active account mismatch: requested ${pubkey}, got ${active}`;
        set({ error: msg, loading: false });
        throw new Error(msg);
      }
      set({ accounts, activeAccount: active, loading: false });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
        loading: false,
      });
      throw err;
    }
  },

  deleteAccount: async (pubkey: string) => {
    try {
      set({ loading: true, error: null });
      await invoke("delete_account", { pubkey });
      const accounts = await invoke<AccountInfo[]>("list_accounts");
      const active = accounts.find((a) => a.active)?.pubkey ?? null;
      set({ accounts, activeAccount: active, loading: false });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
        loading: false,
      });
    }
  },

  renameAccount: async (pubkey: string, label: string) => {
    try {
      set({ loading: true, error: null });
      await invoke("rename_account", { pubkey, label });
      const accounts = await invoke<AccountInfo[]>("list_accounts");
      const active = accounts.find((a) => a.active)?.pubkey ?? null;
      set({ accounts, activeAccount: active, loading: false });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
        loading: false,
      });
    }
  },
}));
