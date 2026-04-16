import { useThreadStore } from "../../stores/threadStore";
import type { ThreadSummary } from "../../stores/serverStore";

interface ThreadIndicatorProps {
  summary: ThreadSummary;
}

export function ThreadIndicator({ summary }: ThreadIndicatorProps) {
  const setActiveThread = useThreadStore((s) => s.setActiveThread);
  const unreadCount = useThreadStore((s) => s.unreadCounts[summary.thread_id] ?? 0);

  return (
    <button
      onClick={() => void setActiveThread(summary.thread_id)}
      className={`mt-2 flex items-center gap-2 rounded-lg border border-ctp-surface1 bg-ctp-mantle p-2 transition hover:bg-ctp-surface0 ${
        unreadCount > 0 ? "ring-1 ring-ctp-blue" : ""
      }`}
    >
      <div className="flex -space-x-1.5 overflow-hidden">
        {/* We'll eventually show participant avatars here */}
        <div className="flex h-5 w-5 items-center justify-center rounded-full bg-ctp-surface1 text-[10px]">
          💬
        </div>
      </div>
      <div className="flex items-center gap-1.5 text-xs font-medium text-ctp-blue">
        <span>{summary.reply_count} {summary.reply_count === 1 ? 'reply' : 'replies'}</span>
        {summary.last_reply_at && (
          <span className="text-ctp-subtext1">
            Last reply {new Date(summary.last_reply_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
          </span>
        )}
      </div>
      {unreadCount > 0 && (
        <div className="flex h-4 min-w-[16px] items-center justify-center rounded-full bg-ctp-blue px-1 text-[10px] text-ctp-crust">
          {unreadCount}
        </div>
      )}
      <div className="ml-1 text-[10px] text-ctp-subtext0 opacity-0 transition-opacity hover:opacity-100">
        View thread →
      </div>
    </button>
  );
}
