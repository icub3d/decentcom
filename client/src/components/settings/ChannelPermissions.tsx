import type { ChannelPermissionOverride, Role } from "../../api/roles";

interface ChannelPermissionsProps {
  roles: Role[];
  overrides: ChannelPermissionOverride[];
  onToggle: (roleId: string, allow: number, deny: number) => Promise<void>;
}

export function ChannelPermissions({ roles, overrides, onToggle }: ChannelPermissionsProps) {
  const byRoleId = new Map(overrides.map((row) => [row.role_id, row]));

  return (
    <div className="space-y-3 rounded-xl border border-ctp-overlay0 bg-ctp-mantle/80 p-4">
      <h3 className="text-sm font-bold text-ctp-text">Channel overrides</h3>
      {roles.map((role) => {
        const overrideRow = byRoleId.get(role.id);
        const allow = overrideRow?.allow ?? 0;
        const deny = overrideRow?.deny ?? 0;

        return (
          <div key={role.id} className="flex items-center justify-between gap-3 text-sm">
            <span className="text-ctp-text">{role.name}</span>
            <div className="flex gap-2">
              <button
                onClick={() => {
                  void onToggle(role.id, allow | 1, deny & ~1);
                }}
                className="rounded-md bg-ctp-green/20 px-2 py-1 text-ctp-green"
              >
                Allow send
              </button>
              <button
                onClick={() => {
                  void onToggle(role.id, allow & ~1, deny | 1);
                }}
                className="rounded-md bg-ctp-red/20 px-2 py-1 text-ctp-red"
              >
                Deny send
              </button>
            </div>
          </div>
        );
      })}
    </div>
  );
}
