import { type KeyboardEvent, useRef, useState } from "react";

import { EmojiPicker } from "../emoji/EmojiPicker";

interface MessageInputProps {
  disabled: boolean;
  onSend: (content: string) => Promise<void>;
}

export function MessageInput({ disabled, onSend }: MessageInputProps) {
  const [value, setValue] = useState("");
  const [sending, setSending] = useState(false);
  const [emojiOpen, setEmojiOpen] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  async function submit() {
    const trimmed = value.trim();
    if (!trimmed || sending || disabled) {
      return;
    }

    setSending(true);
    try {
      await onSend(trimmed);
      setValue("");
    } finally {
      setSending(false);
    }
  }

  async function onKeyDown(event: KeyboardEvent<HTMLTextAreaElement>) {
    if (event.key !== "Enter") {
      return;
    }

    if (event.shiftKey) {
      return;
    }

    event.preventDefault();
    await submit();
  }

  function insertAtCursor(emoji: string) {
    const el = textareaRef.current;
    const start = el?.selectionStart ?? value.length;
    const end = el?.selectionEnd ?? value.length;
    const next = value.slice(0, start) + emoji + value.slice(end);
    setValue(next);
    setEmojiOpen(false);
    requestAnimationFrame(() => {
      if (el) {
        el.selectionStart = el.selectionEnd = start + emoji.length;
        el.focus();
      }
    });
  }

  return (
    <div className="border-t border-ctp-overlay0 p-4 bg-ctp-mantle/90">
      <div className="relative">
        <textarea
          ref={textareaRef}
          value={value}
          onChange={(event) => setValue(event.target.value)}
          onKeyDown={onKeyDown}
          disabled={disabled || sending}
          rows={2}
          placeholder={disabled ? "Connect and select a channel to send" : "Send a message"}
          className="w-full resize-none rounded-lg border border-ctp-overlay0 bg-ctp-base pl-3 pr-24 py-2 text-ctp-text focus:outline-none focus:ring-2 focus:ring-ctp-blue disabled:opacity-60"
        />
        <div className="absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-1">
          {!disabled && (
            <button
              onClick={() => setEmojiOpen((v) => !v)}
              disabled={disabled || sending}
              aria-label="Emoji picker"
              className="rounded-lg px-2 py-1 text-lg transition hover:bg-ctp-surface0 disabled:opacity-60"
            >
              😊
            </button>
          )}
          {emojiOpen && (
            <div className="absolute bottom-full right-0 mb-2">
              <EmojiPicker
                onSelect={insertAtCursor}
                onClose={() => setEmojiOpen(false)}
              />
            </div>
          )}
          <button
            onClick={submit}
            disabled={disabled || sending || !value.trim()}
            className="rounded-lg bg-ctp-blue p-2 text-ctp-crust transition hover:bg-ctp-sapphire disabled:opacity-60"
            aria-label="Send message"
          >
            {sending ? (
              <span className="animate-pulse">...</span>
            ) : (
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="20"
                height="20"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2.5"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <line x1="12" y1="19" x2="12" y2="5"></line>
                <polyline points="5 12 12 5 19 12"></polyline>
              </svg>
            )}
          </button>
        </div>
      </div>
    </div>
  );
}
