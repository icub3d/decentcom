import { render, screen, waitFor } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async () => "pong"),
}));

import App from "./App";

describe("App", () => {
  it("renders and shows ping result", async () => {
    render(<App />);
    expect(screen.getByText("decentcom")).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.getByTestId("ping-result")).toHaveTextContent("pong");
    });
  });
});
