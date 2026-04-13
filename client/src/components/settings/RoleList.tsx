import type { Role } from "../../api/roles";

interface RoleListProps {
  roles: Role[];
  selectedRoleId: string | null;
  onSelectRole: (roleId: string) => void;
}

export function RoleList({ roles, selectedRoleId, onSelectRole }: RoleListProps) {
  return (
    <div className="space-y-2">
      {roles.map((role) => {
        const active = role.id === selectedRoleId;
        return (
          <button
            key={role.id}
            onClick={() => onSelectRole(role.id)}
            className={`w-full rounded-lg border px-3 py-2 text-left text-sm transition ${
              active
                ? "border-ctp-blue bg-ctp-blue/15 text-ctp-blue"
                : "border-ctp-overlay0 bg-ctp-base text-ctp-text hover:bg-ctp-surface0"
            }`}
          >
            <div className="flex items-center justify-between">
              <span className="font-semibold">{role.name}</span>
              <span className="text-xs text-ctp-subtext0">{role.position}</span>
            </div>
          </button>
        );
      })}
    </div>
  );
}
