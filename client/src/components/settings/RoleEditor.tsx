import { useMemo, useState } from "react";

import {
  ADD_REACTIONS,
  ATTACH_FILES,
  BAN_MEMBERS,
  KICK_MEMBERS,
  MANAGE_CHANNELS,
  MANAGE_MESSAGES,
  MANAGE_ROLES,
  READ_MESSAGES,
  SEND_MESSAGES,
} from "../../hooks/usePermissions";

interface RoleEditorProps {
  name: string;
  color: string | null;
  permissions: number;
  onSave: (input: { name: string; color: string | null; permissions: number }) => Promise<void>;
}

const FLAGS: Array<{ bit: number; label: string }> = [
  { bit: SEND_MESSAGES, label: "Send messages" },
  { bit: READ_MESSAGES, label: "Read messages" },
  { bit: MANAGE_MESSAGES, label: "Manage messages" },
  { bit: MANAGE_CHANNELS, label: "Manage channels" },
  { bit: MANAGE_ROLES, label: "Manage roles" },
  { bit: KICK_MEMBERS, label: "Kick members" },
  { bit: BAN_MEMBERS, label: "Ban members" },
  { bit: ATTACH_FILES, label: "Attach files" },
  { bit: ADD_REACTIONS, label: "Add reactions" },
];

function hasFlag(value: number, bit: number) {
  return (value & bit) === bit;
}

function toggleFlag(value: number, bit: number) {
  if (hasFlag(value, bit)) {
    return value & ~bit;
  }
  return value | bit;
}

export function RoleEditor({ name, color, permissions, onSave }: RoleEditorProps) {
  const [draftName, setDraftName] = useState(name);
  const [draftColor, setDraftColor] = useState(color ?? "");
  const [draftPermissions, setDraftPermissions] = useState(permissions);
  const [saving, setSaving] = useState(false);

  const hasChanges = useMemo(() => {
    return (
      draftName !== name ||
      draftColor !== (color ?? "") ||
      draftPermissions !== permissions
    );
  }, [color, draftColor, draftName, draftPermissions, name, permissions]);

  async function save() {
    if (!hasChanges || saving) {
      return;
    }

    setSaving(true);
    try {
      await onSave({
        name: draftName.trim(),
        color: draftColor.trim() || null,
        permissions: draftPermissions,
      });
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="space-y-4 rounded-xl border border-ctp-overlay0 bg-ctp-mantle/80 p-4">
      <div className="space-y-2">
        <label className="block text-xs font-semibold uppercase tracking-wide text-ctp-subtext0">
          Role name
        </label>
        <input
          value={draftName}
          onChange={(event) => setDraftName(event.target.value)}
          className="w-full rounded-lg border border-ctp-overlay0 bg-ctp-base px-3 py-2 text-sm text-ctp-text"
        />
      </div>

      <div className="space-y-2">
        <label className="block text-xs font-semibold uppercase tracking-wide text-ctp-subtext0">
          Color
        </label>
        <input
          value={draftColor}
          onChange={(event) => setDraftColor(event.target.value)}
          placeholder="#aabbcc"
          className="w-full rounded-lg border border-ctp-overlay0 bg-ctp-base px-3 py-2 text-sm text-ctp-text"
        />
      </div>

      <fieldset className="space-y-2">
        <legend className="text-xs font-semibold uppercase tracking-wide text-ctp-subtext0">
          Permissions
        </legend>
        {FLAGS.map((flag) => (
          <label key={flag.bit} className="flex items-center gap-2 text-sm text-ctp-text">
            <input
              type="checkbox"
              checked={hasFlag(draftPermissions, flag.bit)}
              onChange={() => setDraftPermissions((value) => toggleFlag(value, flag.bit))}
            />
            <span>{flag.label}</span>
          </label>
        ))}
      </fieldset>

      <button
        onClick={() => {
          void save();
        }}
        disabled={!hasChanges || saving || draftName.trim().length === 0}
        className="rounded-lg bg-ctp-blue px-4 py-2 text-sm font-bold text-ctp-crust transition hover:bg-ctp-sapphire disabled:opacity-60"
      >
        {saving ? "Saving..." : "Save role"}
      </button>
    </div>
  );
}
