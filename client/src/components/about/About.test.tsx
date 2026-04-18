import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { About } from "./About";

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: vi.fn(),
}));

vi.mock("../ui/Modal", () => ({
  Modal: ({ children, onClose }: { children: React.ReactNode; onClose: () => void }) => (
    <div data-testid="modal">
      <button data-testid="backdrop-close" onClick={onClose}>backdrop</button>
      {children}
    </div>
  ),
}));

describe("About", () => {
  const onClose = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders without crashing", () => {
    render(<About onClose={onClose} />);
    expect(screen.getByText("About decentcom")).toBeInTheDocument();
  });

  it("clicking Close button dismisses the modal", () => {
    render(<About onClose={onClose} />);
    fireEvent.click(screen.getByText("Close"));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("clicking the ✕ button dismisses the modal", () => {
    render(<About onClose={onClose} />);
    fireEvent.click(screen.getByLabelText("Close"));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("shows key strengths", () => {
    render(<About onClose={onClose} />);
    expect(screen.getByText("Identity ownership", { exact: false })).toBeInTheDocument();
    expect(screen.getByText("No passwords", { exact: false })).toBeInTheDocument();
    expect(screen.getByText("Self-hostable", { exact: false })).toBeInTheDocument();
    expect(screen.getByText("Cryptographically secure", { exact: false })).toBeInTheDocument();
    expect(screen.getByText("Open source", { exact: false })).toBeInTheDocument();
  });

  it("renders external links", () => {
    render(<About onClose={onClose} />);
    expect(screen.getByText("Documentation")).toBeInTheDocument();
    expect(screen.getByText("Source Repository")).toBeInTheDocument();
    expect(screen.getByText("Download Page")).toBeInTheDocument();
  });

  it("external links call openUrl instead of navigating", async () => {
    const { openUrl } = await import("@tauri-apps/plugin-opener");
    render(<About onClose={onClose} />);
    fireEvent.click(screen.getByText("Documentation"));
    expect(openUrl).toHaveBeenCalledWith("https://github.com/icub3d/decentcom/wiki");
  });
});
