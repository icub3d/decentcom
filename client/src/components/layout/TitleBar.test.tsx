import { describe, it, expect, vi, beforeEach } from "vitest";

const hoisted = vi.hoisted(() => ({
  isMaximized: vi.fn().mockResolvedValue(false),
  onResized: vi.fn().mockResolvedValue(vi.fn()),
  minimize: vi.fn(),
  toggleMaximize: vi.fn(),
  close: vi.fn(),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => hoisted,
}));

// Import TitleBar AFTER the mock is set up
import { render, screen, fireEvent } from "@testing-library/react";
import { TitleBar } from "./TitleBar";

describe("TitleBar", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders with correct logo", () => {
    render(<TitleBar />);
    expect(screen.getByLabelText("decentcom")).toBeInTheDocument();
  });

  it("calls minimize when minimize button is clicked", () => {
    render(<TitleBar />);
    const minimizeBtn = screen.getByTitle("Minimize");
    fireEvent.click(minimizeBtn);
    expect(hoisted.minimize).toHaveBeenCalled();
  });

  it("calls toggleMaximize when maximize button is clicked", () => {
    render(<TitleBar />);
    const maximizeBtn = screen.getByTitle("Maximize");
    fireEvent.click(maximizeBtn);
    expect(hoisted.toggleMaximize).toHaveBeenCalled();
  });

  it("calls close when close button is clicked", () => {
    render(<TitleBar />);
    const closeBtn = screen.getByTitle("Close");
    fireEvent.click(closeBtn);
    expect(hoisted.close).toHaveBeenCalled();
  });

  it("buttons have data-tauri-no-drag attribute", () => {
    render(<TitleBar />);
    const buttons = screen.getAllByRole("button");
    buttons.forEach(button => {
      expect(button).toHaveAttribute("data-tauri-no-drag");
    });
  });

  it("renders Maximize title when not maximized", async () => {
    hoisted.isMaximized.mockResolvedValue(false);
    render(<TitleBar />);
    expect(screen.getByTitle("Maximize")).toBeInTheDocument();
  });
});
