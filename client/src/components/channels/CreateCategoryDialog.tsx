import { useState } from "react";

interface CreateCategoryDialogProps {
  onClose: () => void;
  onCreate: (name: string, position: number) => Promise<void>;
}

export function CreateCategoryDialog({ onClose, onCreate }: CreateCategoryDialogProps) {
  const [name, setName] = useState("");
  const [position, setPosition] = useState("0");
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

  async function handleSubmit() {
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
      await onCreate(trimmed, pos);
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
        <h3 className="text-sm font-bold text-ctp-text">Create Category</h3>

        <label className="block text-xs font-semibold uppercase tracking-wide text-ctp-subtext0">
          Name
        </label>
        <input
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="Voice Channels"
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
            Create
          </button>
        </div>
      </section>
    </div>
  );
}
