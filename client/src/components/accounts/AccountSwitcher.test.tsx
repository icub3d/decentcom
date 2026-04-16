import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";

import { useIdentityStore } from "../../stores/identityStore";
import { AccountSwitcher } from "./AccountSwitcher";

const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
  save: vi.fn(),
}));

vi.mock("../../services/identity", () => ({
  keyImport: vi.fn(),
  keyBackupReadPubkey: vi.fn(),
  keyExport: vi.fn(),
  keyExportValidatePassphrase: vi.fn(),
}));

const mockGenerateIdentity = vi.fn();
const mockImportIdentity = vi.fn();
const mockRefresh = vi.fn();

vi.mock("../../hooks/useIdentity", () => ({
  useIdentity: () => ({
    hasIdentity: true,
    publicKey: "ABCDEFghij1234567890",
    loading: false,
    error: null,
    generateIdentity: mockGenerateIdentity,
    importIdentity: mockImportIdentity,
    refresh: mockRefresh,
    sign: vi.fn(),
  }),
}));

describe("AccountSwitcher", () => {
  const onSwitch = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockReset();
    mockRefresh.mockReset();
    mockGenerateIdentity.mockReset();
  });

  function seedAccounts() {
    useIdentityStore.setState({
      accounts: [
        { pubkey: "ABCDEFghij1234567890", label: "Alice", active: true },
        { pubkey: "XYZWVUtsrq0987654321", label: null, active: false },
      ],
      activeAccount: "ABCDEFghij1234567890",
      loading: false,
      error: null,
    });
  }

  it("renders account list with labels and short pubkeys", () => {
    seedAccounts();
    render(<AccountSwitcher onSwitchAccount={onSwitch} />);

    expect(screen.getByText(/Alice/)).toBeInTheDocument();
    expect(screen.getByText(/ABCDEF…7890/)).toBeInTheDocument();
    expect(screen.getByText(/XYZWVU…4321/)).toBeInTheDocument();
  });

  it("shows active indicator on current account", () => {
    seedAccounts();
    render(<AccountSwitcher onSwitchAccount={onSwitch} />);

    expect(screen.getByText("●")).toBeInTheDocument();
  });

  it("calls onSwitchAccount when clicking inactive account", () => {
    seedAccounts();
    render(<AccountSwitcher onSwitchAccount={onSwitch} />);

    const inactiveButton = screen.getByText(/XYZWVU…4321/);
    fireEvent.click(inactiveButton);

    expect(onSwitch).toHaveBeenCalledWith("XYZWVUtsrq0987654321");
  });

  it("does not call onSwitchAccount when clicking active account", () => {
    seedAccounts();
    render(<AccountSwitcher onSwitchAccount={onSwitch} />);

    const activeButton = screen.getByText(/Alice/);
    fireEvent.click(activeButton);

    expect(onSwitch).not.toHaveBeenCalled();
  });

  it("opens setup modal when clicking add button", () => {
    seedAccounts();
    mockInvoke.mockResolvedValue(true);
    render(<AccountSwitcher onSwitchAccount={onSwitch} />);

    fireEvent.click(screen.getByText("+ Add Account"));

    expect(screen.getByText("Add Account")).toBeInTheDocument();
    expect(screen.getByText("Create New Identity")).toBeInTheDocument();
  });

  it("closes modal on cancel without calling onSwitchAccount", () => {
    seedAccounts();
    mockInvoke.mockResolvedValue(true);
    render(<AccountSwitcher onSwitchAccount={onSwitch} />);

    fireEvent.click(screen.getByText("+ Add Account"));
    expect(screen.getByText("Add Account")).toBeInTheDocument();

    fireEvent.click(screen.getByText("Cancel"));
    expect(screen.queryByText("Create New Identity")).not.toBeInTheDocument();
    expect(onSwitch).not.toHaveBeenCalled();
  });

  it("closes modal and switches account on successful creation", async () => {
    seedAccounts();
    const newPubkey = "NEWKEYabc12345678900";
    mockGenerateIdentity.mockResolvedValue({
      pubkey: newPubkey,
      seed_phrase: Array(24).fill("word"),
    });
    mockRefresh.mockImplementation(async () => {
      useIdentityStore.setState({
        accounts: [
          { pubkey: "ABCDEFghij1234567890", label: "Alice", active: false },
          { pubkey: newPubkey, label: null, active: true },
        ],
        activeAccount: newPubkey,
      });
    });

    render(<AccountSwitcher onSwitchAccount={onSwitch} />);
    fireEvent.click(screen.getByText("+ Add Account"));
    fireEvent.click(screen.getByText("Create New Identity"));

    await waitFor(() => {
      expect(screen.getByText("Secure Your Identity")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText("I've saved my key — Continue"));

    await waitFor(() => {
      expect(onSwitch).toHaveBeenCalledWith(newPubkey);
    });
  });

  it("shows delete confirmation on trash click", () => {
    seedAccounts();
    render(<AccountSwitcher onSwitchAccount={onSwitch} />);

    const trashButtons = screen.getAllByTitle("Delete account");
    fireEvent.click(trashButtons[0]);

    expect(screen.getByText("Yes")).toBeInTheDocument();
    expect(screen.getByText("No")).toBeInTheDocument();
  });

  it("cancels delete confirmation on No click", () => {
    seedAccounts();
    render(<AccountSwitcher onSwitchAccount={onSwitch} />);

    const trashButtons = screen.getAllByTitle("Delete account");
    fireEvent.click(trashButtons[0]);
    fireEvent.click(screen.getByText("No"));

    expect(screen.queryByText("Yes")).not.toBeInTheDocument();
  });

  it("renders nothing when accounts list is empty", () => {
    useIdentityStore.setState({ accounts: [], activeAccount: null, loading: false, error: null });
    const { container } = render(
      <AccountSwitcher onSwitchAccount={onSwitch} />,
    );
    expect(container.innerHTML).toBe("");
  });
});
