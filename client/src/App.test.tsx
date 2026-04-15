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
});
