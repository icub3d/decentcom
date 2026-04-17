import { useCallback } from "react";

import { useIdentityStore } from "../stores/identityStore";
import { useServerStore } from "../stores/serverStore";
import {
  switchAppStoreAccount,
  initAppStoreForAccount,
  useAppStore,
} from "../stores/appStore";
import { useAccountManagerStore } from "../stores/accountManagerStore";

/**
 * Centralized hook for all account management operations.
 *
 * Consolidates switch / create / import flows that were previously
 * scattered across App.tsx, AccountSwitcher.tsx, and useIdentity.ts.
 * Uses an `isSwitching` flag to serialize operations and prevent
 * auto-connect races during account transitions.
 */
export function useAccountManager() {
  const isSwitching = useAccountManagerStore((s) => s.isSwitching);

  const switchAccount = useCallback(async (pubkey: string) => {
    const { isSwitching: alreadySwitching, setIsSwitching } =
      useAccountManagerStore.getState();
    if (alreadySwitching) {
      throw new Error("Account switch already in progress");
    }

    setIsSwitching(true);
    try {
      // 1. Disconnect from current server (gateway close, no auto-reconnect
      //    because isSwitching is true).
      useServerStore.getState().disconnect();

      // 2. Backend switch first — this ensures auth state is correct before
      //    any localStorage changes.
      const oldPubkey = useIdentityStore.getState().activeAccount;
      await useIdentityStore.getState().setActiveAccount(pubkey);

      // 3. Verify the backend returned the expected active account.
      const confirmedActive = useIdentityStore.getState().activeAccount;
      if (confirmedActive !== pubkey) {
        throw new Error(
          `Backend active account mismatch: expected ${pubkey}, got ${confirmedActive}`,
        );
      }

      // 4. Switch localStorage key and rehydrate the app store for the new
      //    account (awaited — not fire-and-forget).
      switchAppStoreAccount(oldPubkey, pubkey);
    } finally {
      useAccountManagerStore.getState().setIsSwitching(false);
    }
  }, []);

  const handleFirstRunComplete = useCallback(async (refresh: () => Promise<void>) => {
    const { isSwitching: alreadySwitching, setIsSwitching } =
      useAccountManagerStore.getState();
    if (alreadySwitching) return;

    setIsSwitching(true);
    try {
      await refresh();
      const newActive = useIdentityStore.getState().activeAccount;
      if (newActive) {
        // For first-run the active account was just set — init the app store
        // for its localStorage key (await ensures rehydration completes).
        await initAppStoreForAccount(newActive);
      }
    } finally {
      useAccountManagerStore.getState().setIsSwitching(false);
    }
  }, []);

  const handleSetupComplete = useCallback(
    async (
      refresh: () => Promise<void>,
      onSwitchDone?: (pubkey: string) => void,
      preSetupAccount?: string | null,
    ) => {
      const { isSwitching: alreadySwitching, setIsSwitching } =
        useAccountManagerStore.getState();
      if (alreadySwitching) return;

      setIsSwitching(true);
      try {
        // Use the pre-captured account from before the modal opened, because
        // importIdentity already switches the active account on the backend
        // before onComplete fires, so reading getState() here would see the
        // new account and think nothing changed.
        const prevActive = preSetupAccount ?? useIdentityStore.getState().activeAccount;
        await refresh();
        const newActive = useIdentityStore.getState().activeAccount;
        if (newActive && newActive !== prevActive) {
          switchAppStoreAccount(prevActive, newActive);
          onSwitchDone?.(newActive);
        }
      } finally {
        useAccountManagerStore.getState().setIsSwitching(false);
      }
    },
    [],
  );

  /**
   * Whether auto-connect should be suppressed. Intended for use as a
   * guard in the auto-connect effect.
   */
  const shouldAutoConnect = useCallback(
    (opts: {
      hasIdentity: boolean;
      loading: boolean;
      currentServerId: string | null;
      status: string;
      connectLoading: boolean;
    }) => {
      if (useAccountManagerStore.getState().isSwitching) return false;
      return (
        opts.hasIdentity &&
        !opts.loading &&
        !!opts.currentServerId &&
        opts.status === "disconnected" &&
        !opts.connectLoading &&
        useAppStore.persist.hasHydrated()
      );
    },
    [],
  );

  return {
    isSwitching,
    switchAccount,
    handleFirstRunComplete,
    handleSetupComplete,
    shouldAutoConnect,
  };
}
