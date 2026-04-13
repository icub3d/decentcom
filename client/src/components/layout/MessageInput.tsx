import { KeyboardEvent, useState } from "react";

interface MessageInputProps {
  disabled: boolean;
  onSend: (content: string) => Promise<void>;
}

export function MessageInput({ disabled, onSend }: MessageInputProps) {
  const [value, setValue] = useState("");
  const [sending, setSending] = useState(false);

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

  return (
    <div className="border-t border-ctp-overlay0 p-4 bg-ctp-mantle/90">
      <div className="flex gap-3">
        <textarea
          value={value}
          onChange={(event) => setValue(event.target.value)}
          onKeyDown={onKeyDown}
          disabled={disabled || sending}
          rows={2}
          placeholder={disabled ? "Connect and select a channel to send" : "Send a message"}
          className="flex-1 resize-none rounded-lg border border-ctp-overlay0 bg-ctp-base px-3 py-2 text-ctp-text focus:outline-none focus:ring-2 focus:ring-ctp-blue disabled:opacity-60"
        />
        <button
          onClick={submit}
          disabled={disabled || sending || !value.trim()}
          className="rounded-lg bg-ctp-blue px-4 py-2 text-sm font-bold text-ctp-crust transition hover:bg-ctp-sapphire disabled:opacity-60"
        >
          {sending ? "..." : "Send"}
        </button>
      </div>
    </div>
  );
}
