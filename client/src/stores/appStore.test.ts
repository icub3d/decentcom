import { describe, it, expect, beforeEach, vi } from "vitest";
import { useAppStore } from "./appStore";

vi.mock("../theme/apply", () => ({
  applyTheme: vi.fn(),
  defaultTheme: () => "mocha",
}));

beforeEach(() => {
  useAppStore.setState({ currentServerId: null, servers: {}, theme: "mocha" });
});

describe("getStateForBackup", () => {
  it("returns version 1 snapshot of servers and theme", () => {
    useAppStore.getState().addServer("http://localhost:3000", "Local");
    useAppStore.setState({ theme: "latte" });

    const backup = useAppStore.getState().getStateForBackup();

    expect(backup.version).toBe(1);
    expect(backup.theme).toBe("latte");
    expect(Object.keys(backup.servers)).toHaveLength(1);
    expect(backup.servers["http://localhost:3000"].name).toBe("Local");
  });

  it("returns empty servers when store has no servers", () => {
    const backup = useAppStore.getState().getStateForBackup();
    expect(backup.servers).toEqual({});
  });
});

describe("loadFromBackup", () => {
  it("replaces servers and theme from backup state", () => {
    useAppStore.getState().addServer("http://old:3000", "Old");

    useAppStore.getState().loadFromBackup({
      version: 1,
      servers: {
        "http://new:3000": {
          id: "http://new:3000",
          address: "http://new:3000",
          name: "New",
        },
      },
      theme: "frappe",
    });

    const state = useAppStore.getState();
    expect(state.servers).toEqual({
      "http://new:3000": {
        id: "http://new:3000",
        address: "http://new:3000",
        name: "New",
      },
    });
    expect(state.theme).toBe("frappe");
    expect(state.currentServerId).toBeNull();
  });

  it("roundtrip: getStateForBackup then loadFromBackup preserves data", () => {
    useAppStore.getState().addServer("http://server:3000", "Test Server");
    useAppStore.setState({ theme: "macchiato" });

    const snapshot = useAppStore.getState().getStateForBackup();

    useAppStore.setState({ servers: {}, theme: "mocha" });
    useAppStore.getState().loadFromBackup(snapshot);

    const state = useAppStore.getState();
    expect(state.theme).toBe("macchiato");
    expect(state.servers["http://server:3000"].name).toBe("Test Server");
  });
});
