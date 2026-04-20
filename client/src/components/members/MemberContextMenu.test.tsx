import { describe, it, expect, vi, beforeEach } from "vitest";

const hoisted = vi.hoisted(() => ({
  permissionBits: 0,
}));

vi.mock("../../hooks/usePermissions", async (importOriginal) => {
  const actual = (await importOriginal()) as Record<string, unknown>;
  return {
    ...actual,
    usePermissions: () => ({
      bits: hoisted.permissionBits,
      has: (perm: number) => (hoisted.permissionBits & perm) === perm,
    }),
  };
});

import { render, screen, fireEvent } from "@testing-library/react";
import { MemberContextMenu } from "./MemberContextMenu";
import { KICK_MEMBERS, BAN_MEMBERS, MANAGE_ROLES } from "../../hooks/usePermissions";
import type { Member } from "../../api/members";
import type { Role } from "../../api/roles";

const baseMember: Member = {
  user_id: "u1",
  pubkey: "pk1",
  display_name: "Alice",
  avatar_hash: null,
  is_bot: false,
  joined_at: "2025-01-01T00:00:00Z",
  roles: [],
};

const mockRoles: Role[] = [
  { id: "everyone", name: "@everyone", color: null, permissions: 0, position: 0, is_builtin: true, created_at: "", updated_at: "" },
  { id: "admin", name: "@admin", color: null, permissions: 0, position: 100, is_builtin: true, created_at: "", updated_at: "" },
  { id: "mod", name: "Moderator", color: "#ff0", permissions: 0, position: 50, is_builtin: false, created_at: "", updated_at: "" },
  { id: "vip", name: "VIP", color: "#0ff", permissions: 0, position: 10, is_builtin: false, created_at: "", updated_at: "" },
];

const noop = async () => {};

describe("MemberContextMenu", () => {
  beforeEach(() => {
    hoisted.permissionBits = 0;
    vi.clearAllMocks();
  });

  it("renders nothing when user has no permissions", () => {
    const { container } = render(
      <MemberContextMenu
        member={baseMember}
        roles={mockRoles}
        memberRoleIds={[]}
        onKick={noop}
        onBan={noop}
        onAssignRole={noop}
        onRemoveRole={noop}
      />,
    );
    expect(container.innerHTML).toBe("");
  });

  it("renders ⋯ button and no inline Kick/Ban buttons", () => {
    hoisted.permissionBits = KICK_MEMBERS | BAN_MEMBERS;
    render(
      <MemberContextMenu
        member={baseMember}
        roles={mockRoles}
        memberRoleIds={[]}
        onKick={noop}
        onBan={noop}
        onAssignRole={noop}
        onRemoveRole={noop}
      />,
    );
    expect(screen.getByLabelText("Member actions")).toBeInTheDocument();
    // No inline Kick/Ban buttons visible before opening menu
    expect(screen.queryByText("Kick")).not.toBeInTheDocument();
    expect(screen.queryByText("Ban")).not.toBeInTheDocument();
  });

  it("opens menu on ⋯ click and closes on outside click", () => {
    hoisted.permissionBits = KICK_MEMBERS;
    render(
      <MemberContextMenu
        member={baseMember}
        roles={mockRoles}
        memberRoleIds={[]}
        onKick={noop}
        onBan={noop}
        onAssignRole={noop}
        onRemoveRole={noop}
      />,
    );
    fireEvent.click(screen.getByLabelText("Member actions"));
    expect(screen.getByText("Kick")).toBeInTheDocument();

    // Outside click closes menu
    fireEvent.mouseDown(document.body);
    expect(screen.queryByText("Kick")).not.toBeInTheDocument();
  });

  it("hides Kick/Ban items when user lacks corresponding permission", () => {
    // Only MANAGE_ROLES, no kick or ban
    hoisted.permissionBits = MANAGE_ROLES;
    render(
      <MemberContextMenu
        member={baseMember}
        roles={mockRoles}
        memberRoleIds={[]}
        onKick={noop}
        onBan={noop}
        onAssignRole={noop}
        onRemoveRole={noop}
      />,
    );
    fireEvent.click(screen.getByLabelText("Member actions"));
    expect(screen.queryByText("Kick")).not.toBeInTheDocument();
    expect(screen.queryByText("Ban")).not.toBeInTheDocument();
    // But roles section is present
    expect(screen.getByText("Roles")).toBeInTheDocument();
  });

  it("hides role toggle section when user lacks MANAGE_ROLES", () => {
    hoisted.permissionBits = KICK_MEMBERS | BAN_MEMBERS;
    render(
      <MemberContextMenu
        member={baseMember}
        roles={mockRoles}
        memberRoleIds={[]}
        onKick={noop}
        onBan={noop}
        onAssignRole={noop}
        onRemoveRole={noop}
      />,
    );
    fireEvent.click(screen.getByLabelText("Member actions"));
    expect(screen.getByText("Kick")).toBeInTheDocument();
    expect(screen.getByText("Ban")).toBeInTheDocument();
    expect(screen.queryByText("Roles")).not.toBeInTheDocument();
    expect(screen.queryByText("Moderator")).not.toBeInTheDocument();
  });

  it("shows role toggles with checkmarks for assigned roles", () => {
    hoisted.permissionBits = MANAGE_ROLES;
    render(
      <MemberContextMenu
        member={baseMember}
        roles={mockRoles}
        memberRoleIds={["mod"]}
        onKick={noop}
        onBan={noop}
        onAssignRole={noop}
        onRemoveRole={noop}
      />,
    );
    fireEvent.click(screen.getByLabelText("Member actions"));
    // Non-builtin roles appear
    expect(screen.getByText("Moderator")).toBeInTheDocument();
    expect(screen.getByText("VIP")).toBeInTheDocument();
    // Builtin roles do not appear as toggles
    expect(screen.queryByText("@everyone")).not.toBeInTheDocument();
    expect(screen.queryByText("@admin")).not.toBeInTheDocument();
    // Checkmark for assigned role
    expect(screen.getByText("✓")).toBeInTheDocument();
  });

  it("shows kick confirmation on Kick click", () => {
    hoisted.permissionBits = KICK_MEMBERS;
    render(
      <MemberContextMenu
        member={baseMember}
        roles={mockRoles}
        memberRoleIds={[]}
        onKick={noop}
        onBan={noop}
        onAssignRole={noop}
        onRemoveRole={noop}
      />,
    );
    fireEvent.click(screen.getByLabelText("Member actions"));
    fireEvent.click(screen.getByText("Kick"));
    expect(screen.getByText("Kick this member?")).toBeInTheDocument();
    expect(screen.getByText("Confirm")).toBeInTheDocument();
    expect(screen.getByText("Cancel")).toBeInTheDocument();
  });

  it("shows ban reason input on Ban click", () => {
    hoisted.permissionBits = BAN_MEMBERS;
    render(
      <MemberContextMenu
        member={baseMember}
        roles={mockRoles}
        memberRoleIds={[]}
        onKick={noop}
        onBan={noop}
        onAssignRole={noop}
        onRemoveRole={noop}
      />,
    );
    fireEvent.click(screen.getByLabelText("Member actions"));
    fireEvent.click(screen.getByText("Ban"));
    expect(screen.getByText("Ban this member?")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("Reason (optional)")).toBeInTheDocument();
  });
});
