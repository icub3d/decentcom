import { useState } from "react";
import { createPortal } from "react-dom";

const NEW_CATEGORY_VALUE = "__new__";
const CHANNEL_NAME_RE = /^[a-z0-9]([a-z0-9-]*[a-z0-9])?$/;

interface CreateChannelDialogProps {
  existingCategories: string[];
  onClose: () => void;
  onCreate: (
    name: string,
    category: string | null,
    position: number,
  ) => Promise<void>;
}

export function CreateChannelDialog({
  existingCategories,
  onClose,
  onCreate,
}: CreateChannelDialogProps) {
  const [name, setName] = useState("");
  const [categorySelection, setCategorySelection] = useState("");
  const [newCategoryName, setNewCategoryName] = useState("");
  const [position, setPosition] = useState("0");
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

  const creatingCategory = categorySelection === NEW_CATEGORY_VALUE;

  async function handleSubmit() {
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

    if (creatingCategory && !newCategoryName.trim()) {
      setError("New category name is required.");
      return;
    }

    const resolvedCategory = creatingCategory
      ? newCategoryName.trim()
      : categorySelection || null;

    setPending(true);
    try {
      await onCreate(trimmed, resolvedCategory, pos);
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
        <h3 className="text-sm font-bold text-ctp-text">Create Channel</h3>

        <label className="block text-xs font-semibold uppercase tracking-wide text-ctp-subtext0">
          Name
        </label>
        <input
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="general"
          autoFocus
          className="w-full rounded-lg border border-ctp-overlay0 bg-ctp-base px-3 py-2 text-sm text-ctp-text"
        />

        <label className="block text-xs font-semibold uppercase tracking-wide text-ctp-subtext0">
          Category
        </label>
        <select
          value={categorySelection}
          onChange={(e) => setCategorySelection(e.target.value)}
          className="w-full rounded-lg border border-ctp-overlay0 bg-ctp-base px-3 py-2 text-sm text-ctp-text"
        >
          <option value="">None (uncategorized)</option>
          {existingCategories.map((cat) => (
            <option key={cat} value={cat}>
              {cat}
            </option>
          ))}
          <option value={NEW_CATEGORY_VALUE}>+ New category…</option>
        </select>

        {creatingCategory && (
          <>
            <label className="block text-xs font-semibold uppercase tracking-wide text-ctp-subtext0">
              New Category Name
            </label>
            <input
              value={newCategoryName}
              onChange={(e) => setNewCategoryName(e.target.value)}
              placeholder="Voice Channels"
              className="w-full rounded-lg border border-ctp-overlay0 bg-ctp-base px-3 py-2 text-sm text-ctp-text"
            />
          </>
        )}

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

        {error && (
          <p className="rounded-lg border border-ctp-red bg-ctp-red/20 px-3 py-2 text-xs text-ctp-red">
            {error}
          </p>
        )}

        <div className="flex gap-2 justify-end">
          <button
            onClick={onClose}
            className="rounded-lg border border-ctp-overlay0 px-4 py-2 text-sm text-ctp-subtext1 hover:bg-ctp-surface0"
          >
            Cancel
          </button>
          <button
            disabled={pending}
            onClick={() => {
              void handleSubmit();
            }}
            className="rounded-lg bg-ctp-blue px-4 py-2 text-sm font-bold text-ctp-crust transition hover:bg-ctp-sapphire disabled:opacity-60"
          >
            {pending ? "Creating…" : "Create"}
          </button>
        </div>
      </section>
    </div>,
    document.body,
  );
}
