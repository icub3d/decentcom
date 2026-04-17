import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { Setup } from "./Setup";

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
  save: vi.fn(),
}));

vi.mock("../services/identity", () => ({
  keyImport: vi.fn(),
  keyBackupReadPubkey: vi.fn(),
  keyExport: vi.fn(),
  keyExportValidatePassphrase: vi.fn(),
}));

import { open } from "@tauri-apps/plugin-dialog";
import { keyImport, keyBackupReadPubkey } from "../services/identity";

const fakeIdentity = {
  pubkey: "ABC123DEF456",
  seed_phrase: Array.from({ length: 24 }, (_, i) => `word${i + 1}`),
};

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

  it("successful backup import without metadata calls onComplete", async () => {
    vi.mocked(open).mockResolvedValueOnce("/tmp/test.dckb");
    vi.mocked(keyBackupReadPubkey).mockResolvedValueOnce({
      pubkey: "ABC123DEF456",
    });
    vi.mocked(keyImport).mockResolvedValueOnce({
      pubkey: "ABC123DEF456",
      metadata: null,
    });

    render(
      <Setup
        onGenerate={onGenerate}
        onImport={onImport}
        onComplete={onComplete}
      />,
    );

    fireEvent.click(screen.getByText("Restore from Backup File"));
    fireEvent.click(screen.getByText(/Choose .dckb file/));
    await waitFor(() => {
      expect(screen.getByText(/ABC123/)).toBeInTheDocument();
    });

    const input = screen.getByPlaceholderText("Passphrase");
    fireEvent.change(input, { target: { value: "my-passphrase" } });

    fireEvent.click(screen.getByText("Restore Identity"));

    await waitFor(() => {
      expect(keyImport).toHaveBeenCalledWith("my-passphrase", "/tmp/test.dckb");
      expect(onComplete).toHaveBeenCalled();
    });
  });

  it("backup import with metadata shows settings-restored confirmation", async () => {
    vi.mocked(open).mockResolvedValueOnce("/tmp/test.dckb");
    vi.mocked(keyBackupReadPubkey).mockResolvedValueOnce({
      pubkey: "ABC123DEF456",
    });
    vi.mocked(keyImport).mockResolvedValueOnce({
      pubkey: "ABC123DEF456",
      metadata: JSON.stringify({
        version: 1,
        servers: {
          "http://open:3000": {
            id: "http://open:3000",
            address: "http://open:3000",
            name: "Open Server",
          },
        },
        theme: "frappe",
      }),
    });

    render(
      <Setup
        onGenerate={onGenerate}
        onImport={onImport}
        onComplete={onComplete}
      />,
    );

    fireEvent.click(screen.getByText("Restore from Backup File"));
    fireEvent.click(screen.getByText(/Choose .dckb file/));
    await waitFor(() => {
      expect(screen.getByText(/ABC123/)).toBeInTheDocument();
    });

    const input = screen.getByPlaceholderText("Passphrase");
    fireEvent.change(input, { target: { value: "my-passphrase" } });
    fireEvent.click(screen.getByText("Restore Identity"));

    await waitFor(() => {
      expect(screen.getByText("Settings Restored")).toBeInTheDocument();
    });
    expect(screen.getByText("1")).toBeInTheDocument();
    expect(screen.getByText("frappe")).toBeInTheDocument();
    expect(onComplete).not.toHaveBeenCalled();

    fireEvent.click(screen.getByText("Continue"));
    expect(onComplete).toHaveBeenCalledTimes(1);
  });

  it("after clicking Create New Identity, the Key Safety screen is rendered", async () => {
    onGenerate.mockResolvedValueOnce(fakeIdentity);

    render(
      <Setup
        onGenerate={onGenerate}
        onImport={onImport}
        onComplete={onComplete}
      />,
    );

    fireEvent.click(screen.getByText("Create New Identity"));

    await waitFor(() => {
      expect(screen.getByText("Secure Your Identity")).toBeInTheDocument();
    });
    expect(onComplete).not.toHaveBeenCalled();
  });

  it("onComplete is not called until the user clicks confirm on the safety screen", async () => {
    onGenerate.mockResolvedValueOnce(fakeIdentity);

    render(
      <Setup
        onGenerate={onGenerate}
        onImport={onImport}
        onComplete={onComplete}
      />,
    );

    fireEvent.click(screen.getByText("Create New Identity"));

    await waitFor(() => {
      expect(screen.getByText("Secure Your Identity")).toBeInTheDocument();
    });
    expect(onComplete).not.toHaveBeenCalled();

    fireEvent.click(screen.getByText(/I've saved my key/));
    expect(onComplete).toHaveBeenCalledTimes(1);
  });

  it("seed phrase words are displayed correctly after reveal", async () => {
    onGenerate.mockResolvedValueOnce(fakeIdentity);

    render(
      <Setup
        onGenerate={onGenerate}
        onImport={onImport}
        onComplete={onComplete}
      />,
    );

    fireEvent.click(screen.getByText("Create New Identity"));

    await waitFor(() => {
      expect(screen.getByText("Secure Your Identity")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText("Reveal Recovery Phrase"));

    for (let i = 0; i < 24; i++) {
      expect(screen.getByText(`word${i + 1}`)).toBeInTheDocument();
      expect(screen.getByText(`${i + 1}.`)).toBeInTheDocument();
    }
  });

  it("copy button copies the seed phrase to clipboard", async () => {
    onGenerate.mockResolvedValueOnce(fakeIdentity);
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, { clipboard: { writeText } });

    render(
      <Setup
        onGenerate={onGenerate}
        onImport={onImport}
        onComplete={onComplete}
      />,
    );

    fireEvent.click(screen.getByText("Create New Identity"));

    await waitFor(() => {
      expect(screen.getByText("Secure Your Identity")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText("Reveal Recovery Phrase"));
    fireEvent.click(screen.getByText("Copy to Clipboard"));

    await waitFor(() => {
      expect(writeText).toHaveBeenCalledWith(
        fakeIdentity.seed_phrase.join(" "),
      );
      expect(screen.getByText("✓ Copied!")).toBeInTheDocument();
    });
  });
});
