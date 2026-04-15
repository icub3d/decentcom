import { useRef, useState } from "react";
import { deleteAvatar, updateProfile, uploadAvatar } from "../../api/profile";
import { useServerStore } from "../../stores/serverStore";
import { useMembersStore } from "../../stores/members";
import { Avatar } from "./Avatar";

export function ProfileEditor() {
  const address = useServerStore((s) => s.address);
  const token = useServerStore((s) => s.sessionToken);
  const userId = useServerStore((s) => s.sessionUserId);
  const member = useMembersStore((s) => s.members.find((m) => m.user_id === userId));
  const updateMemberProfile = useMembersStore((s) => s.updateMemberProfile);

  const [displayName, setDisplayName] = useState(member?.display_name ?? "");
  const [saving, setSaving] = useState(false);
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);

  async function handleSaveDisplayName() {
    if (!token || !userId) return;
    setSaving(true);
    setError(null);
    setSuccess(null);
    try {
      const trimmed = displayName.trim();
      const updated = await updateProfile(address, token, {
        display_name: trimmed || undefined,
      });
      updateMemberProfile(userId, {
        display_name: updated.display_name ?? null,
      });
      setSuccess("Display name updated");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSaving(false);
    }
  }

  async function handleUploadAvatar(file: File) {
    if (!token || !userId) return;
    setUploading(true);
    setError(null);
    setSuccess(null);
    try {
      const updated = await uploadAvatar(address, token, file);
      updateMemberProfile(userId, {
        avatar_hash: updated.avatar_hash ?? null,
      });
      setSuccess("Avatar updated");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setUploading(false);
    }
  }

  async function handleDeleteAvatar() {
    if (!token || !userId) return;
    setUploading(true);
    setError(null);
    setSuccess(null);
    try {
      await deleteAvatar(address, token);
      updateMemberProfile(userId, { avatar_hash: null });
      setSuccess("Avatar removed");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setUploading(false);
    }
  }

  const pubkey = member?.pubkey ?? "";

  return (
    <section className="rounded-xl border border-ctp-overlay0 bg-ctp-mantle/90 p-4">
      <h3 className="mb-3 text-sm font-bold text-ctp-text">My Profile</h3>

      <div className="mb-4 flex items-center gap-3">
        <Avatar pubkey={pubkey} avatarHash={member?.avatar_hash} size={48} />
        <div className="flex gap-2">
          <button
            onClick={() => fileRef.current?.click()}
            disabled={uploading}
            className="rounded-md bg-ctp-blue px-3 py-1 text-xs font-semibold text-ctp-crust hover:bg-ctp-blue/80 disabled:opacity-50"
          >
            {uploading ? "Uploading…" : "Change Avatar"}
          </button>
          {member?.avatar_hash && (
            <button
              onClick={() => void handleDeleteAvatar()}
              disabled={uploading}
              className="rounded-md border border-ctp-overlay0 px-3 py-1 text-xs text-ctp-subtext1 hover:bg-ctp-surface0 disabled:opacity-50"
            >
              Remove
            </button>
          )}
        </div>
        <input
          ref={fileRef}
          type="file"
          accept="image/jpeg,image/png,image/webp"
          className="hidden"
          onChange={(e) => {
            const file = e.target.files?.[0];
            if (file) void handleUploadAvatar(file);
            e.target.value = "";
          }}
        />
      </div>

      <div className="mb-3">
        <label className="mb-1 block text-xs font-medium text-ctp-subtext1">Display Name</label>
        <div className="flex gap-2">
          <input
            type="text"
            value={displayName}
            onChange={(e) => setDisplayName(e.target.value)}
            maxLength={32}
            placeholder="Enter a display name"
            className="flex-1 rounded-md border border-ctp-overlay0 bg-ctp-base px-3 py-1.5 text-sm text-ctp-text placeholder:text-ctp-overlay1 focus:border-ctp-blue focus:outline-none"
          />
          <button
            onClick={() => void handleSaveDisplayName()}
            disabled={saving}
            className="rounded-md bg-ctp-blue px-3 py-1.5 text-xs font-semibold text-ctp-crust hover:bg-ctp-blue/80 disabled:opacity-50"
          >
            {saving ? "Saving…" : "Save"}
          </button>
        </div>
      </div>

      {error && <p className="text-xs text-ctp-red">{error}</p>}
      {success && <p className="text-xs text-ctp-green">{success}</p>}
    </section>
  );
}
