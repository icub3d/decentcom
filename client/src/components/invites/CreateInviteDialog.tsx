import { useMemo, useState } from "react";

import type { Role } from "../../api/roles";

interface CreateInviteDialogProps {
  roles: Role[];
  loading?: boolean;
  onCreate: (input: {
    max_uses?: number;
    expires_in_seconds?: number;
    grant_role_id?: string;
  }) => Promise<{ invite_link: string }>;
}

export function CreateInviteDialog({ roles, loading = false, onCreate }: CreateInviteDialogProps) {
  const [maxUses, setMaxUses] = useState("0");
  const [expiresHours, setExpiresHours] = useState("");
  const [grantRoleId, setGrantRoleId] = useState("");
  const [createdLink, setCreatedLink] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const roleOptions = useMemo(() => roles.filter((role) => !role.is_builtin), [roles]);

  async function handleCreate() {
    setError(null);
    try {
      const parsedMaxUses = Number(maxUses);
      if (!Number.isInteger(parsedMaxUses) || parsedMaxUses < 0) {
        setError("Max uses must be a non-negative integer.");
        return;
      }

      const parsedHours = expiresHours.trim() ? Number(expiresHours) : null;
      if (parsedHours !== null && (!Number.isFinite(parsedHours) || parsedHours <= 0)) {
        setError("Expiry hours must be greater than zero.");
        return;
      }

      const created = await onCreate({
        max_uses: parsedMaxUses,
        expires_in_seconds: parsedHours ? Math.round(parsedHours * 3600) : undefined,
        grant_role_id: grantRoleId || undefined,
      });
      setCreatedLink(created.invite_link);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  async function copyLink() {
    if (!createdLink) {
      return;
    }
    await navigator.clipboard.writeText(createdLink);
  }

  return (
    <section className="rounded-xl border border-ctp-overlay0 bg-ctp-mantle/80 p-4 space-y-3">
      <h3 className="text-sm font-bold text-ctp-text">Create Invite</h3>

      <label className="block text-xs font-semibold uppercase tracking-wide text-ctp-subtext0">
        Max Uses (0 = unlimited)
      </label>
      <input
        value={maxUses}
        onChange={(event) => setMaxUses(event.target.value)}
        className="w-full rounded-lg border border-ctp-overlay0 bg-ctp-base px-3 py-2 text-sm text-ctp-text"
      />

      <label className="block text-xs font-semibold uppercase tracking-wide text-ctp-subtext0">
        Expiry (hours, optional)
      </label>
      <input
        value={expiresHours}
        onChange={(event) => setExpiresHours(event.target.value)}
        placeholder="24"
        className="w-full rounded-lg border border-ctp-overlay0 bg-ctp-base px-3 py-2 text-sm text-ctp-text"
      />

      <label className="block text-xs font-semibold uppercase tracking-wide text-ctp-subtext0">
        Grant Role (optional)
      </label>
      <select
        value={grantRoleId}
        onChange={(event) => setGrantRoleId(event.target.value)}
        className="w-full rounded-lg border border-ctp-overlay0 bg-ctp-base px-3 py-2 text-sm text-ctp-text"
      >
        <option value="">None</option>
        {roleOptions.map((role) => (
          <option key={role.id} value={role.id}>
            {role.name}
          </option>
        ))}
      </select>

      {error && (
        <p className="rounded-lg border border-ctp-red bg-ctp-red/20 px-3 py-2 text-xs text-ctp-red">
          {error}
        </p>
      )}

      <button
        disabled={loading}
        onClick={() => {
          void handleCreate();
        }}
        className="rounded-lg bg-ctp-blue px-4 py-2 text-sm font-bold text-ctp-crust transition hover:bg-ctp-sapphire disabled:opacity-60"
      >
        Generate Invite
      </button>

      {createdLink && (
        <div className="rounded-lg border border-ctp-overlay0 bg-ctp-base p-3 text-xs text-ctp-subtext1 space-y-2">
          <div className="break-all">{createdLink}</div>
          <button
            onClick={() => {
              void copyLink();
            }}
            className="rounded-md bg-ctp-green/20 px-3 py-1 font-semibold text-ctp-green"
          >
            Copy Link
          </button>
        </div>
      )}
    </section>
  );
}
