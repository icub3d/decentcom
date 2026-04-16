import { useRef, useState } from "react";

import type { ReactionSummary } from "../../api/reactions";
import { putReaction } from "../../api/reactions";
import { useServerStore } from "../../stores/serverStore";
import { EmojiPicker } from "../emoji/EmojiPicker";

interface ReactionBarProps {
  channelId: string;
  messageId: string;
  reactions: ReactionSummary[];
}

export function ReactionBar({ channelId, messageId, reactions }: ReactionBarProps) {
  const [pickerOpen, setPickerOpen] = useState(false);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const address = useServerStore((s) => s.address);
  const token = useServerStore((s) => s.sessionToken);

  async function handleReactionClick(emoji: string) {
    if (!token) return;
    try {
      await putReaction(address, token, channelId, messageId, emoji);
    } catch {
      // Ignore — gateway event will reconcile state
    }
  }

  async function handlePickerSelect(emoji: string) {
    setPickerOpen(false);
    await handleReactionClick(emoji);
  }

  return (
    <div className="relative mt-1.5 flex flex-wrap items-center gap-1">
      {reactions.map((r) => (
        <button
          key={r.emoji}
          onClick={() => void handleReactionClick(r.emoji)}
          title={`${r.emoji} · ${r.count}`}
          className={`flex items-center gap-1 rounded-full border px-2 py-0.5 text-sm transition ${
            r.me
              ? "border-ctp-blue bg-ctp-blue/20 text-ctp-blue hover:bg-ctp-blue/30"
              : "border-ctp-overlay0 bg-ctp-surface0 text-ctp-text hover:bg-ctp-surface1"
          }`}
        >
          <span>{r.emoji}</span>
          <span className="text-xs font-medium">{r.count}</span>
        </button>
      ))}

      <div>
        <button
          ref={triggerRef}
          onClick={() => setPickerOpen((v) => !v)}
          aria-label="Add reaction"
          className="flex items-center gap-1 rounded-full border border-ctp-overlay0 px-2 py-0.5 text-sm text-ctp-subtext0 transition hover:border-ctp-blue hover:bg-ctp-surface1 hover:text-ctp-text"
        >
          <span>😊</span>
          <span className="text-xs">+</span>
        </button>
        {pickerOpen && (
          <EmojiPicker
            anchorRef={triggerRef}
            onSelect={(emoji) => void handlePickerSelect(emoji)}
            onClose={() => setPickerOpen(false)}
          />
        )}
      </div>
    </div>
  );
}
