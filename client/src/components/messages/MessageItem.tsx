import type { Message } from "../../stores/serverStore";

interface MessageItemProps {
  message: Message;
}

function formatTime(timestamp: string): string {
  return new Date(timestamp).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function MessageItem({ message }: MessageItemProps) {
  return (
    <article className="rounded-lg border border-ctp-overlay0 bg-ctp-mantle/60 px-4 py-3">
      <div className="mb-1 flex items-center gap-2 text-xs text-ctp-subtext0">
        <span className="font-semibold text-ctp-subtext1">{message.author_id}</span>
        <time>{formatTime(message.created_at)}</time>
        {message.edited_at && !message.deleted && <span className="text-ctp-yellow">(edited)</span>}
      </div>
      {message.deleted ? (
        <p className="italic text-ctp-overlay1">This message was deleted.</p>
      ) : (
        <p className="whitespace-pre-wrap text-ctp-text">{message.content}</p>
      )}
    </article>
  );
}
