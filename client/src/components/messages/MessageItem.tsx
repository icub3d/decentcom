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
    <article className="rounded-lg border border-slate-800 bg-slate-900/60 px-4 py-3">
      <div className="mb-1 flex items-center gap-2 text-xs text-slate-400">
        <span className="font-semibold text-slate-300">{message.author_id}</span>
        <time>{formatTime(message.created_at)}</time>
        {message.edited_at && !message.deleted && <span className="text-amber-300">(edited)</span>}
      </div>
      {message.deleted ? (
        <p className="italic text-slate-500">This message was deleted.</p>
      ) : (
        <p className="whitespace-pre-wrap text-slate-100">{message.content}</p>
      )}
    </article>
  );
}
