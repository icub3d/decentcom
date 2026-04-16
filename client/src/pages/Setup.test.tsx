import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { Setup } from "./Setup";

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

vi.mock("../services/identity", () => ({
  keyImport: vi.fn(),
  keyBackupReadPubkey: vi.fn(),
}));

import { open } from "@tauri-apps/plugin-dialog";
import { keyImport, keyBackupReadPubkey } from "../services/identity";

describe("Setup", () => {
  const onGenerate = vi.fn();
  const onImport = vi.fn();
  const onComplete = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders the "Restore from Backup File" button on the choice view', () => {
    render(
      <Setup
        onGenerate={onGenerate}
        onImport={onImport}
        onComplete={onComplete}
      />,
    );
    expect(
      screen.getByText("Restore from Backup File"),
    ).toBeInTheDocument();
  });

  it('clicking "Restore from Backup File" transitions to the backup import view', async () => {
    render(
      <Setup
        onGenerate={onGenerate}
        onImport={onImport}
        onComplete={onComplete}
      />,
    );
    fireEvent.click(screen.getByText("Restore from Backup File"));
    expect(screen.getByText("Restore from Backup")).toBeInTheDocument();
    expect(screen.getByText(/Choose .dckb file/)).toBeInTheDocument();
  });

  it("back button from backup view returns to choice view", () => {
    render(
      <Setup
        onGenerate={onGenerate}
        onImport={onImport}
        onComplete={onComplete}
      />,
    );
    fireEvent.click(screen.getByText("Restore from Backup File"));
    expect(screen.getByText("Restore from Backup")).toBeInTheDocument();

    fireEvent.click(screen.getByText("Back"));
    expect(screen.getByText("Welcome")).toBeInTheDocument();
    expect(
      screen.getByText("Restore from Backup File"),
    ).toBeInTheDocument();
  });

  it("successful backup import calls onComplete", async () => {
    vi.mocked(open).mockResolvedValueOnce("/tmp/test.dckb");
    vi.mocked(keyBackupReadPubkey).mockResolvedValueOnce({
      pubkey: "ABC123DEF456",
    });
    vi.mocked(keyImport).mockResolvedValueOnce({ pubkey: "ABC123DEF456" });

    render(
      <Setup
        onGenerate={onGenerate}
        onImport={onImport}
        onComplete={onComplete}
      />,
    );

    // Navigate to backup view
    fireEvent.click(screen.getByText("Restore from Backup File"));

    // Pick file
    fireEvent.click(screen.getByText(/Choose .dckb file/));
    await waitFor(() => {
      expect(screen.getByText(/ABC123/)).toBeInTheDocument();
    });

    // Enter passphrase
    const input = screen.getByPlaceholderText("Passphrase");
    fireEvent.change(input, { target: { value: "my-passphrase" } });

    // Click restore
    fireEvent.click(screen.getByText("Restore Identity"));

    await waitFor(() => {
      expect(keyImport).toHaveBeenCalledWith("my-passphrase", "/tmp/test.dckb");
      expect(onComplete).toHaveBeenCalled();
    });
  });
});
