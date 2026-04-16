import { act } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";

const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

import { useIdentityStore } from "./identityStore";

describe("identityStore", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    // Reset store state between tests.
    useIdentityStore.setState({
      accounts: [],
      activeAccount: null,
      loading: false,
      error: null,
    });
  });

  it("refresh populates accounts and active account", async () => {
    mockInvoke.mockResolvedValueOnce([
      { pubkey: "pk-alice", label: "Alice", active: true },
      { pubkey: "pk-bob", label: null, active: false },
    ]);

    await act(async () => {
      await useIdentityStore.getState().refresh();
    });

    const state = useIdentityStore.getState();
    expect(state.accounts).toHaveLength(2);
    expect(state.activeAccount).toBe("pk-alice");
    expect(state.loading).toBe(false);
    expect(state.error).toBeNull();
    expect(mockInvoke).toHaveBeenCalledWith("list_accounts");
  });

  it("refresh sets error on failure", async () => {
    mockInvoke.mockRejectedValueOnce(new Error("keychain locked"));

    await act(async () => {
      await useIdentityStore.getState().refresh();
    });

    const state = useIdentityStore.getState();
    expect(state.error).toBe("keychain locked");
    expect(state.loading).toBe(false);
  });

  it("setActiveAccount calls backend and refreshes list", async () => {
    // First call: set_active_account, second: list_accounts
    mockInvoke
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce([
        { pubkey: "pk-alice", label: "Alice", active: false },
        { pubkey: "pk-bob", label: null, active: true },
      ]);

    await act(async () => {
      await useIdentityStore.getState().setActiveAccount("pk-bob");
    });

    expect(mockInvoke).toHaveBeenCalledWith("set_active_account", { pubkey: "pk-bob" });
    expect(useIdentityStore.getState().activeAccount).toBe("pk-bob");
  });

  it("deleteAccount calls backend and refreshes list", async () => {
    mockInvoke
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce([
        { pubkey: "pk-alice", label: "Alice", active: true },
      ]);

    await act(async () => {
      await useIdentityStore.getState().deleteAccount("pk-bob");
    });

    expect(mockInvoke).toHaveBeenCalledWith("delete_account", { pubkey: "pk-bob" });
    expect(useIdentityStore.getState().accounts).toHaveLength(1);
  });

  it("renameAccount calls backend and refreshes list", async () => {
    mockInvoke
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce([
        { pubkey: "pk-alice", label: "Alice Renamed", active: true },
      ]);

    await act(async () => {
      await useIdentityStore.getState().renameAccount("pk-alice", "Alice Renamed");
    });

    expect(mockInvoke).toHaveBeenCalledWith("rename_account", { pubkey: "pk-alice", label: "Alice Renamed" });
    expect(useIdentityStore.getState().accounts[0].label).toBe("Alice Renamed");
  });
});
