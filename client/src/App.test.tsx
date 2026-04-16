import { render, screen, waitFor } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (command: string) => {
    if (command === "has_identity") return false;
    if (command === "ping") return "pong";
    return null;
  }),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    isMaximized: vi.fn().mockResolvedValue(false),
    onResized: vi.fn().mockResolvedValue(vi.fn()),
    minimize: vi.fn(),
    toggleMaximize: vi.fn(),
    close: vi.fn(),
  }),
}));

import App from "./App";

describe("App", () => {
  it("renders setup screen when no identity exists", async () => {
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText("Welcome")).toBeInTheDocument();
      expect(screen.getByText("Create New Identity")).toBeInTheDocument();
    });
  });

  it("does not render Setup full-screen when hasIdentity is true", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation(async (command: string) => {
      if (command === "has_identity") return true;
      if (command === "get_public_key") return { pubkey: "testkey123" };
      if (command === "list_accounts")
        return [{ pubkey: "testkey123", label: null, active: true }];
      if (command === "ping") return "pong";
      return null;
    });

    render(<App />);
    await waitFor(() => {
      expect(screen.queryByText("Welcome")).not.toBeInTheDocument();
      expect(screen.queryByText("Create New Identity")).not.toBeInTheDocument();
    });
  });
});
