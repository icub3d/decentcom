import {
  type DragEvent,
  type KeyboardEvent,
  useCallback,
  useRef,
  useState,
} from "react";

import type { Attachment } from "../../api/media";
import { uploadFile } from "../../api/media";
import { useServerStore } from "../../stores/serverStore";
import { EmojiPicker } from "../emoji/EmojiPicker";

interface MessageInputProps {
  disabled: boolean;
  onSend: (content: string, attachmentIds?: string[]) => Promise<void>;
}

interface PendingFile {
  id: string;
  name: string;
  status: "uploading" | "ready" | "error";
  attachment: Attachment | null;
  error: string | null;
}

let nextPendingId = 0;

export function MessageInput({ disabled, onSend }: MessageInputProps) {
  const [value, setValue] = useState("");
  const [sending, setSending] = useState(false);
  const [emojiOpen, setEmojiOpen] = useState(false);
  const [pendingFiles, setPendingFiles] = useState<PendingFile[]>([]);
  const [dragOver, setDragOver] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const emojiButtonRef = useRef<HTMLButtonElement>(null);

  const address = useServerStore((s) => s.address);
  const sessionToken = useServerStore((s) => s.sessionToken);
  const currentChannelId = useServerStore((s) => s.currentChannelId);

  const uploadFiles = useCallback(
    (files: File[]) => {
      if (!sessionToken || !currentChannelId) return;

      for (const file of files) {
        const id = `pending-${++nextPendingId}`;
        setPendingFiles((prev) => [
          ...prev,
          { id, name: file.name, status: "uploading", attachment: null, error: null },
        ]);

        uploadFile(address, sessionToken, currentChannelId, file)
          .then((attachment) => {
            setPendingFiles((prev) =>
              prev.map((p) =>
                p.id === id ? { ...p, status: "ready" as const, attachment } : p,
              ),
            );
          })
          .catch((err) => {
            setPendingFiles((prev) =>
              prev.map((p) =>
                p.id === id
                  ? { ...p, status: "error" as const, error: err instanceof Error ? err.message : "upload failed" }
                  : p,
              ),
            );
          });
      }
    },
    [address, sessionToken, currentChannelId],
  );

  const hasContent = value.trim().length > 0;
  const readyAttachments = pendingFiles.filter((p) => p.status === "ready");
  const uploading = pendingFiles.some((p) => p.status === "uploading");
  const canSend =
    !uploading && (hasContent || readyAttachments.length > 0);

  async function submit() {
    if (!canSend || sending || disabled) return;

    setSending(true);
    try {
      const attachmentIds = readyAttachments.map((p) => p.attachment!.id);
      await onSend(value.trim(), attachmentIds.length > 0 ? attachmentIds : undefined);
      setValue("");
      setPendingFiles([]);
    } finally {
      setSending(false);
    }
  }

  async function onKeyDown(event: KeyboardEvent<HTMLTextAreaElement>) {
    if (event.key !== "Enter") return;
    if (event.shiftKey) return;
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

  function removePending(id: string) {
    setPendingFiles((prev) => prev.filter((p) => p.id !== id));
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    setDragOver(true);
  }

  function handleDragLeave(e: DragEvent) {
    e.preventDefault();
    setDragOver(false);
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    setDragOver(false);
    if (disabled) return;
    const files = Array.from(e.dataTransfer.files);
    if (files.length > 0) uploadFiles(files);
  }

  function handleFileSelect() {
    const input = fileInputRef.current;
    if (!input?.files) return;
    uploadFiles(Array.from(input.files));
    input.value = "";
  }

  return (
    <div
      className="border-t border-ctp-overlay0 p-4 bg-ctp-mantle/90"
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      {/* Pending files display */}
      {pendingFiles.length > 0 && (
        <div className="mb-2 flex flex-wrap gap-2">
          {pendingFiles.map((p) => (
            <div
              key={p.id}
              className="flex items-center gap-2 rounded-lg border border-ctp-overlay0 bg-ctp-surface0/50 px-2 py-1 text-xs"
            >
              <span className="max-w-[120px] truncate text-ctp-text">
                {p.name}
              </span>
              {p.status === "error" ? (
                <span className="text-ctp-red" title={p.error ?? ""}>{p.error}</span>
              ) : p.status === "ready" ? (
                <span className="text-ctp-green">✓</span>
              ) : (
                <span className="animate-pulse text-ctp-subtext0">uploading…</span>
              )}
              <button
                onClick={() => removePending(p.id)}
                className="text-ctp-overlay1 hover:text-ctp-red transition"
                aria-label={`Remove ${p.name}`}
              >
                ×
              </button>
            </div>
          ))}
        </div>
      )}

      <div className={`relative ${dragOver ? "ring-2 ring-ctp-blue rounded-lg" : ""}`}>
        <input
          ref={fileInputRef}
          type="file"
          multiple
          className="hidden"
          onChange={handleFileSelect}
        />
        <textarea
          ref={textareaRef}
          value={value}
          onChange={(event) => setValue(event.target.value)}
          onKeyDown={onKeyDown}
          disabled={disabled || sending}
          rows={2}
          placeholder={disabled ? "Connect and select a channel to send" : "Send a message"}
          className="w-full resize-none rounded-lg border border-ctp-overlay0 bg-ctp-base pl-10 pr-24 py-2 text-ctp-text focus:outline-none focus:ring-2 focus:ring-ctp-blue disabled:opacity-60"
        />
        {/* Attach file button (left side) */}
        <div className="absolute left-2 top-1/2 -translate-y-1/2">
          {!disabled && (
            <button
              onClick={() => fileInputRef.current?.click()}
              disabled={disabled || sending}
              aria-label="Attach file"
              className="rounded-lg p-1 text-ctp-subtext0 transition hover:bg-ctp-surface0 hover:text-ctp-text disabled:opacity-60"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="20"
                height="20"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <path d="m21.44 11.05-9.19 9.19a6 6 0 0 1-8.49-8.49l8.57-8.57A4 4 0 1 1 18 8.84l-8.59 8.57a2 2 0 0 1-2.83-2.83l8.49-8.48" />
              </svg>
            </button>
          )}
        </div>
        {/* Right side controls */}
        <div className="absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-1">
          {!disabled && (
            <button
              ref={emojiButtonRef}
              onClick={() => setEmojiOpen((v) => !v)}
              disabled={disabled || sending}
              aria-label="Emoji picker"
              className="rounded-lg px-2 py-1 text-lg transition hover:bg-ctp-surface0 disabled:opacity-60"
            >
              😊
            </button>
          )}
          {emojiOpen && (
            <EmojiPicker
              anchorRef={emojiButtonRef}
              onSelect={insertAtCursor}
              onClose={() => setEmojiOpen(false)}
            />
          )}
          <button
            onClick={submit}
            disabled={disabled || sending || !canSend}
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

