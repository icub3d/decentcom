import { useState } from "react";

import type { Category } from "../../stores/serverStore";

interface EditCategoryDialogProps {
  category: Category;
  onClose: () => void;
  onUpdate: (categoryId: string, name: string, position: number) => Promise<void>;
  onDelete: (categoryId: string) => Promise<void>;
}

export function EditCategoryDialog({ category, onClose, onUpdate, onDelete }: EditCategoryDialogProps) {
  const [name, setName] = useState(category.name);
  const [position, setPosition] = useState(String(category.position));
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);

  async function handleSave() {
    setError(null);
    const trimmed = name.trim();

    if (!trimmed) {
      setError("Category name is required.");
      return;
    }

    const pos = Number(position);
    if (!Number.isInteger(pos) || pos < 0) {
      setError("Position must be a non-negative integer.");
      return;
    }

    setPending(true);
    try {
      await onUpdate(category.id, trimmed, pos);
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
      await onDelete(category.id);
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setPending(false);
    }
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-ctp-crust/60"
      onClick={onClose}
    >
      <section
        className="w-full max-w-md rounded-xl border border-ctp-overlay0 bg-ctp-mantle p-5 space-y-3"
        onClick={(e) => e.stopPropagation()}
      >
        <h3 className="text-sm font-bold text-ctp-text">Edit Category</h3>

        <label className="block text-xs font-semibold uppercase tracking-wide text-ctp-subtext0">
          Name
        </label>
        <input
          value={name}
          onChange={(e) => setName(e.target.value)}
          className="w-full rounded-lg border border-ctp-overlay0 bg-ctp-base px-3 py-2 text-sm text-ctp-text"
        />

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

        {confirmDelete && (
          <p className="rounded-lg border border-ctp-yellow bg-ctp-yellow/10 px-3 py-2 text-xs text-ctp-yellow">
            Deleting this category will move its channels to uncategorized.
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
    </div>
  );
}
