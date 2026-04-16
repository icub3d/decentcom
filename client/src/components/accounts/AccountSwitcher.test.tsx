import { render, screen, fireEvent } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";

import { useIdentityStore } from "../../stores/identityStore";
import { AccountSwitcher } from "./AccountSwitcher";

const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe("AccountSwitcher", () => {
  const onSwitch = vi.fn();
  const onAdd = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockReset();
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
    render(<AccountSwitcher onSwitchAccount={onSwitch} onAddAccount={onAdd} />);

    expect(screen.getByText(/Alice/)).toBeInTheDocument();
    expect(screen.getByText(/ABCDEF…7890/)).toBeInTheDocument();
    // Unlabeled account shows short pubkey only
    expect(screen.getByText(/XYZWVU…4321/)).toBeInTheDocument();
  });

  it("shows active indicator on current account", () => {
    seedAccounts();
    render(<AccountSwitcher onSwitchAccount={onSwitch} onAddAccount={onAdd} />);

    // The active account has a green dot
    expect(screen.getByText("●")).toBeInTheDocument();
  });

  it("calls onSwitchAccount when clicking inactive account", () => {
    seedAccounts();
    render(<AccountSwitcher onSwitchAccount={onSwitch} onAddAccount={onAdd} />);

    const inactiveButton = screen.getByText(/XYZWVU…4321/);
    fireEvent.click(inactiveButton);

    expect(onSwitch).toHaveBeenCalledWith("XYZWVUtsrq0987654321");
  });

  it("does not call onSwitchAccount when clicking active account", () => {
    seedAccounts();
    render(<AccountSwitcher onSwitchAccount={onSwitch} onAddAccount={onAdd} />);

    const activeButton = screen.getByText(/Alice/);
    fireEvent.click(activeButton);

    expect(onSwitch).not.toHaveBeenCalled();
  });

  it("calls onAddAccount when clicking add button", () => {
    seedAccounts();
    render(<AccountSwitcher onSwitchAccount={onSwitch} onAddAccount={onAdd} />);

    fireEvent.click(screen.getByText("+ Add Account"));
    expect(onAdd).toHaveBeenCalled();
  });

  it("shows delete confirmation on trash click", () => {
    seedAccounts();
    render(<AccountSwitcher onSwitchAccount={onSwitch} onAddAccount={onAdd} />);

    const trashButtons = screen.getAllByTitle("Delete account");
    fireEvent.click(trashButtons[0]);

    expect(screen.getByText("Yes")).toBeInTheDocument();
    expect(screen.getByText("No")).toBeInTheDocument();
  });

  it("cancels delete confirmation on No click", () => {
    seedAccounts();
    render(<AccountSwitcher onSwitchAccount={onSwitch} onAddAccount={onAdd} />);

    const trashButtons = screen.getAllByTitle("Delete account");
    fireEvent.click(trashButtons[0]);
    fireEvent.click(screen.getByText("No"));

    expect(screen.queryByText("Yes")).not.toBeInTheDocument();
  });

  it("renders nothing when accounts list is empty", () => {
    useIdentityStore.setState({ accounts: [], activeAccount: null, loading: false, error: null });
    const { container } = render(
      <AccountSwitcher onSwitchAccount={onSwitch} onAddAccount={onAdd} />,
    );
    expect(container.innerHTML).toBe("");
  });
});
