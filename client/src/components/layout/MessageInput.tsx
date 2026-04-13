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
    <div className="border-t border-slate-800 p-4 bg-slate-900/90">
      <div className="flex gap-3">
        <textarea
          value={value}
          onChange={(event) => setValue(event.target.value)}
          onKeyDown={onKeyDown}
          disabled={disabled || sending}
          rows={2}
          placeholder={disabled ? "Connect and select a channel to send" : "Send a message"}
          className="flex-1 resize-none rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-slate-100 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-60"
        />
        <button
          onClick={submit}
          disabled={disabled || sending || !value.trim()}
          className="rounded-lg bg-blue-600 px-4 py-2 text-sm font-bold text-white transition hover:bg-blue-500 disabled:opacity-60"
        >
          {sending ? "..." : "Send"}
        </button>
      </div>
    </div>
  );
}
