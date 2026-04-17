import { create } from "zustand";

/**
 * Lightweight store that tracks whether an account switch is in progress.
 * Kept as a standalone zustand store (rather than React state) so that
 * non-React code (e.g. auto-connect guards, serverStore) can read the
 * flag synchronously via `useAccountManagerStore.getState().isSwitching`.
 */
interface AccountManagerState {
  isSwitching: boolean;
  setIsSwitching: (v: boolean) => void;
}

export const useAccountManagerStore = create<AccountManagerState>()((set) => ({
  isSwitching: false,
  setIsSwitching: (v) => set({ isSwitching: v }),
}));
