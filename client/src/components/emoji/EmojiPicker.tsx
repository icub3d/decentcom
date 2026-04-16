import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";

import { EMOJI_CATEGORIES, EMOJIS, type EmojiCategory } from "../../data/emojis";

interface EmojiPickerProps {
  /** The trigger element used to position the picker above it. */
  anchorRef: React.RefObject<HTMLElement | null>;
  onSelect: (emoji: string) => void;
  onClose: () => void;
}

const CATEGORY_ICONS: Record<EmojiCategory, string> = {
  "Smileys & Emotion": "😊",
  "People & Body": "👋",
  "Animals & Nature": "🐱",
  "Food & Drink": "🍕",
  "Travel & Places": "🚀",
  Activities: "🎮",
  Objects: "💻",
  Symbols: "✅",
};

const PICKER_WIDTH = 288; // w-72

export function EmojiPicker({ anchorRef, onSelect, onClose }: EmojiPickerProps) {
  const [activeCategory, setActiveCategory] = useState<EmojiCategory>(EMOJI_CATEGORIES[0]);
  const pickerRef = useRef<HTMLDivElement>(null);

  const filtered = EMOJIS.filter((e) => e.category === activeCategory);

  // Compute position from anchor; falls back to 0,0 in test environments where anchor is null.
  const rect = anchorRef.current?.getBoundingClientRect() ?? null;
  const top = rect ? rect.top - 8 : 0;
  const left = rect ? rect.right - PICKER_WIDTH : 0;

  // Close on outside click
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      const target = event.target as Node;
      const outsidePicker = !pickerRef.current?.contains(target);
      const outsideAnchor = !anchorRef.current?.contains(target);
      if (outsidePicker && outsideAnchor) {
        onClose();
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [onClose, anchorRef]);

  // Close on Escape
  useEffect(() => {
    function handleEscape(event: KeyboardEvent) {
      if (event.key === "Escape") {
        onClose();
      }
    }
    document.addEventListener("keydown", handleEscape);
    return () => document.removeEventListener("keydown", handleEscape);
  }, [onClose]);

  return createPortal(
    <div
      ref={pickerRef}
      style={{ position: "fixed", top: `${top}px`, left: `${left}px`, transform: "translateY(-100%)" }}
      className="z-50 w-72 rounded-lg border border-ctp-surface1 bg-ctp-surface0 shadow-lg"
      role="dialog"
      aria-label="Emoji picker"
    >
      {/* Category tabs */}
      <div className="flex border-b border-ctp-overlay0 px-1 pt-1" role="tablist">
        {EMOJI_CATEGORIES.map((cat) => (
          <button
            key={cat}
            role="tab"
            aria-selected={activeCategory === cat}
            aria-label={cat}
            onClick={() => setActiveCategory(cat)}
            className={`flex-1 rounded-t-md px-1 py-1.5 text-center text-sm transition ${
              activeCategory === cat
                ? "bg-ctp-surface1 text-ctp-text"
                : "text-ctp-subtext0 hover:bg-ctp-surface1/50"
            }`}
          >
            {CATEGORY_ICONS[cat]}
          </button>
        ))}
      </div>

      {/* Emoji grid */}
      <div className="h-48 overflow-y-auto p-2">
        <div className="grid grid-cols-8 gap-0.5">
          {filtered.map((entry) => (
            <button
              key={entry.label}
              onClick={() => onSelect(entry.emoji)}
              title={entry.label}
              className="rounded p-1 text-lg hover:bg-ctp-surface1 active:bg-ctp-overlay0"
            >
              {entry.emoji}
            </button>
          ))}
        </div>
      </div>
    </div>,
    document.body,
  );
}
