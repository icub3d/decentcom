import { describe, expect, it, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";

import { useAccountManagerStore } from "../stores/accountManagerStore";
import { useIdentityStore } from "../stores/identityStore";

// Mock Tauri invoke
const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

// Mock serverStore
const mockDisconnect = vi.fn();
vi.mock("../stores/serverStore", () => ({
  useServerStore: Object.assign(vi.fn(() => ({ status: "disconnected" })), {
    getState: () => ({
      disconnect: mockDisconnect,
      status: "disconnected",
      connect: vi.fn(),
    }),
  }),
}));

// Mock appStore
const mockSwitchAppStoreAccount = vi.fn();
const mockInitAppStoreForAccount = vi.fn().mockResolvedValue(undefined);
vi.mock("../stores/appStore", () => ({
  switchAppStoreAccount: (...args: unknown[]) => mockSwitchAppStoreAccount(...args),
  initAppStoreForAccount: (...args: unknown[]) => mockInitAppStoreForAccount(...args),
  useAppStore: Object.assign(vi.fn(() => ({})), {
    persist: { hasHydrated: () => true },
    getState: () => ({ currentServerId: null, servers: {}, theme: "mocha" }),
  }),
}));

import { useAccountManager } from "./useAccountManager";

describe("useAccountManager", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockReset();
    mockDisconnect.mockReset();
    mockSwitchAppStoreAccount.mockReset();
    mockInitAppStoreForAccount.mockReset().mockResolvedValue(undefined);
    useAccountManagerStore.setState({ isSwitching: false });
    useIdentityStore.setState({
      accounts: [
        { pubkey: "pubkey-A", label: "Alice", active: true },
        { pubkey: "pubkey-B", label: "Bob", active: false },
      ],
      activeAccount: "pubkey-A",
      loading: false,
      error: null,
    });
  });

  describe("switchAccount", () => {
    it("sets isSwitching, calls backend, switches app store, clears flag", async () => {
      // Mock setActiveAccount to simulate backend success
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "set_active_account") return undefined;
        if (cmd === "list_accounts") {
          return [
            { pubkey: "pubkey-A", label: "Alice", active: false },
            { pubkey: "pubkey-B", label: "Bob", active: true },
          ];
        }
      });

      const { result } = renderHook(() => useAccountManager());

      expect(result.current.isSwitching).toBe(false);

      await act(async () => {
        await result.current.switchAccount("pubkey-B");
      });

      // Verify sequence: disconnect, backend, then appStore switch
      expect(mockDisconnect).toHaveBeenCalledTimes(1);
      expect(mockInvoke).toHaveBeenCalledWith("set_active_account", { pubkey: "pubkey-B" });
      expect(mockSwitchAppStoreAccount).toHaveBeenCalledWith("pubkey-A", "pubkey-B");

      // isSwitching should be cleared
      expect(result.current.isSwitching).toBe(false);
      expect(useAccountManagerStore.getState().isSwitching).toBe(false);
    });

    it("rejects concurrent calls while isSwitching is true", async () => {
      // Set up a slow backend call
      let resolveSwitch: () => void;
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "set_active_account") {
          return new Promise<void>((resolve) => {
            resolveSwitch = resolve;
          });
        }
        if (cmd === "list_accounts") {
          return Promise.resolve([
            { pubkey: "pubkey-A", label: "Alice", active: false },
            { pubkey: "pubkey-B", label: "Bob", active: true },
          ]);
        }
      });

      const { result } = renderHook(() => useAccountManager());

      // Start first switch (will be pending)
      let firstDone = false;
      const firstSwitch = act(async () => {
        await result.current.switchAccount("pubkey-B");
        firstDone = true;
      });

      // While first is in progress, try a second
      await expect(
        act(async () => {
          await result.current.switchAccount("pubkey-A");
        }),
      ).rejects.toThrow("Account switch already in progress");

      // Complete the first switch
      resolveSwitch!();
      await firstSwitch;
      expect(firstDone).toBe(true);
    });

    it("throws if backend returns different active account", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "set_active_account") return undefined;
        if (cmd === "list_accounts") {
          // Backend returns wrong active account
          return [
            { pubkey: "pubkey-A", label: "Alice", active: true },
            { pubkey: "pubkey-B", label: "Bob", active: false },
          ];
        }
      });

      const { result } = renderHook(() => useAccountManager());

      let caughtError: Error | null = null;
      await act(async () => {
        try {
          await result.current.switchAccount("pubkey-B");
        } catch (e) {
          caughtError = e as Error;
        }
      });

      expect(caughtError?.message).toMatch(/Backend active account mismatch/);

      // isSwitching should still be cleared on error
      expect(useAccountManagerStore.getState().isSwitching).toBe(false);
    });
  });

  describe("shouldAutoConnect", () => {
    it("returns false when isSwitching is true", () => {
      useAccountManagerStore.setState({ isSwitching: true });
      const { result } = renderHook(() => useAccountManager());

      const canConnect = result.current.shouldAutoConnect({
        hasIdentity: true,
        loading: false,
        currentServerId: "http://localhost:8080",
        status: "disconnected",
        connectLoading: false,
        address: "http://localhost:8080",
      });

      expect(canConnect).toBe(false);
    });

    it("returns true when conditions are met and not switching", () => {
      useAccountManagerStore.setState({ isSwitching: false });
      const { result } = renderHook(() => useAccountManager());

      const canConnect = result.current.shouldAutoConnect({
        hasIdentity: true,
        loading: false,
        currentServerId: "http://localhost:8080",
        status: "disconnected",
        connectLoading: false,
        address: "http://localhost:8080",
      });

      expect(canConnect).toBe(true);
    });

    it("returns true when currentServerId is different from current address", () => {
      useAccountManagerStore.setState({ isSwitching: false });
      const { result } = renderHook(() => useAccountManager());

      const canConnect = result.current.shouldAutoConnect({
        hasIdentity: true,
        loading: false,
        currentServerId: "http://localhost:8081",
        status: "connected",
        connectLoading: false,
        address: "http://localhost:8080",
      });

      expect(canConnect).toBe(true);
    });

    it("returns false when no currentServerId", () => {
      const { result } = renderHook(() => useAccountManager());

      const canConnect = result.current.shouldAutoConnect({
        hasIdentity: true,
        loading: false,
        currentServerId: null,
        status: "disconnected",
        connectLoading: false,
        address: "",
      });

      expect(canConnect).toBe(false);
    });
  });

  describe("handleFirstRunComplete", () => {
    it("calls refresh and inits app store for new account", async () => {
      const mockRefresh = vi.fn().mockImplementation(async () => {
        useIdentityStore.setState({ activeAccount: "new-pubkey" });
      });

      const { result } = renderHook(() => useAccountManager());

      await act(async () => {
        await result.current.handleFirstRunComplete(mockRefresh);
      });

      expect(mockRefresh).toHaveBeenCalledTimes(1);
      expect(mockInitAppStoreForAccount).toHaveBeenCalledWith("new-pubkey");
      expect(result.current.isSwitching).toBe(false);
    });
  });

  describe("handleSetupComplete", () => {
    it("calls refresh, switches app store, and invokes callback", async () => {
      const onSwitchDone = vi.fn();
      const mockRefresh = vi.fn().mockImplementation(async () => {
        useIdentityStore.setState({ activeAccount: "pubkey-B" });
      });

      const { result } = renderHook(() => useAccountManager());

      await act(async () => {
        await result.current.handleSetupComplete(mockRefresh, onSwitchDone);
      });

      expect(mockRefresh).toHaveBeenCalledTimes(1);
      expect(mockSwitchAppStoreAccount).toHaveBeenCalledWith("pubkey-A", "pubkey-B");
      expect(onSwitchDone).toHaveBeenCalledWith("pubkey-B");
      expect(result.current.isSwitching).toBe(false);
    });

    it("uses preSetupAccount when import already switched backend active account", async () => {
      // Simulate: importIdentity already changed activeAccount to pubkey-B
      // before handleSetupComplete runs, so getState() would return pubkey-B.
      useIdentityStore.setState({ activeAccount: "pubkey-B" });

      const onSwitchDone = vi.fn();
      const mockRefresh = vi.fn().mockResolvedValue(undefined);

      const { result } = renderHook(() => useAccountManager());

      await act(async () => {
        // Pass preSetupAccount="pubkey-A" (captured before modal opened)
        await result.current.handleSetupComplete(mockRefresh, onSwitchDone, "pubkey-A");
      });

      expect(mockSwitchAppStoreAccount).toHaveBeenCalledWith("pubkey-A", "pubkey-B");
      expect(onSwitchDone).toHaveBeenCalledWith("pubkey-B");
    });
  });
});
