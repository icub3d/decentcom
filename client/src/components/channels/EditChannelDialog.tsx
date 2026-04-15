import { useState } from "react";
import { createPortal } from "react-dom";

import type { Category, Channel } from "../../stores/serverStore";

interface EditChannelDialogProps {
  channel: Channel;
  categories: Category[];
  onClose: () => void;
  onUpdate: (channelId: string, name: string, categoryId: string | null, position: number, topic: string | null) => Promise<void>;
  onDelete: (channelId: string) => Promise<void>;
}

const CHANNEL_NAME_RE = /^[a-z0-9]([a-z0-9-]*[a-z0-9])?$/;

export function EditChannelDialog({ channel, categories, onClose, onUpdate, onDelete }: EditChannelDialogProps) {
  const [name, setName] = useState(channel.name);
  const [categoryId, setCategoryId] = useState(channel.category_id ?? "");
  const [position, setPosition] = useState(String(channel.position));
  const [topic, setTopic] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);

  async function handleSave() {
    setError(null);
    const trimmed = name.trim().toLowerCase();

    if (!trimmed) {
      setError("Channel name is required.");
      return;
    }
    if (!CHANNEL_NAME_RE.test(trimmed)) {
      setError("Channel name must be lowercase alphanumeric with hyphens only (no spaces).");
      return;
    }

    const pos = Number(position);
    if (!Number.isInteger(pos) || pos < 0) {
      setError("Position must be a non-negative integer.");
      return;
    }

    setPending(true);
    try {
      await onUpdate(channel.id, trimmed, categoryId || null, pos, topic || null);
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setPending(false);
    }
  }

  async function handleDelete() {
    setPending(true);
    try {
      await onDelete(channel.id);
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setPending(false);
    }
  }

  return createPortal(
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-ctp-crust/60"
      onMouseDown={onClose}
    >
      <section
        className="w-full max-w-md rounded-xl border border-ctp-overlay0 bg-ctp-mantle p-5 space-y-3"
        onMouseDown={(e) => e.stopPropagation()}
      >
        <h3 className="text-sm font-bold text-ctp-text">Edit Channel</h3>

        <label className="block text-xs font-semibold uppercase tracking-wide text-ctp-subtext0">
          Name
        </label>
        <input
          value={name}
          onChange={(e) => setName(e.target.value)}
          className="w-full rounded-lg border border-ctp-overlay0 bg-ctp-base px-3 py-2 text-sm text-ctp-text"
        />

        <label className="block text-xs font-semibold uppercase tracking-wide text-ctp-subtext0">
          Category
        </label>
        <select
          value={categoryId}
          onChange={(e) => setCategoryId(e.target.value)}
          className="w-full rounded-lg border border-ctp-overlay0 bg-ctp-base px-3 py-2 text-sm text-ctp-text"
        >
          <option value="">None</option>
          {categories.map((cat) => (
            <option key={cat.id} value={cat.id}>
              {cat.name}
            </option>
          ))}
        </select>

        <label className="block text-xs font-semibold uppercase tracking-wide text-ctp-subtext0">
          Position
        </label>
        <input
          type="number"
          value={position}
          onChange={(e) => setPosition(e.target.value)}
          min="0"
          className="w-full rounded-lg border border-ctp-overlay0 bg-ctp-base px-3 py-2 text-sm text-ctp-text"
        />

        <label className="block text-xs font-semibold uppercase tracking-wide text-ctp-subtext0">
          Topic (optional)
        </label>
        <input
          value={topic}
          onChange={(e) => setTopic(e.target.value)}
          placeholder="What this channel is about"
          className="w-full rounded-lg border border-ctp-overlay0 bg-ctp-base px-3 py-2 text-sm text-ctp-text"
        />

        {error && (
          <p className="rounded-lg border border-ctp-red bg-ctp-red/20 px-3 py-2 text-xs text-ctp-red">
            {error}
          </p>
        )}

        <div className="flex items-center justify-between">
          {!confirmDelete ? (
            <button
              onClick={() => setConfirmDelete(true)}
              disabled={pending}
              className="rounded-md border border-ctp-red/50 bg-ctp-red/10 px-3 py-2 text-xs text-ctp-red hover:bg-ctp-red/20 disabled:opacity-60"
            >
              Delete
            </button>
          ) : (
            <div className="flex gap-2">
              <button
                onClick={() => {
                  void handleDelete();
                }}
                disabled={pending}
                className="rounded-md bg-ctp-red px-3 py-2 text-xs font-bold text-ctp-crust hover:bg-ctp-red/80 disabled:opacity-60"
              >
                Confirm Delete
              </button>
              <button
                onClick={() => setConfirmDelete(false)}
                className="rounded-md border border-ctp-overlay0 px-3 py-2 text-xs text-ctp-subtext1 hover:bg-ctp-surface0"
              >
                Cancel
              </button>
            </div>
          )}

          <div className="flex gap-2">
            <button
              onClick={onClose}
              className="rounded-lg border border-ctp-overlay0 px-4 py-2 text-sm text-ctp-subtext1 hover:bg-ctp-surface0"
            >
              Cancel
            </button>
            <button
              disabled={pending}
              onClick={() => {
                void handleSave();
              }}
              className="rounded-lg bg-ctp-blue px-4 py-2 text-sm font-bold text-ctp-crust transition hover:bg-ctp-sapphire disabled:opacity-60"
            >
              Save
            </button>
          </div>
        </div>
      </section>
    </div>,
    document.body,
  );
}
