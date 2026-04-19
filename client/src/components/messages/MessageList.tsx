import { useEffect, useMemo, useRef } from "react";

import type { Message } from "../../stores/serverStore";
import { MessageItem } from "./MessageItem";

interface MessageListProps {
  messages: Message[];
  hasMore: boolean;
  loading: boolean;
  onLoadMore: () => Promise<void>;
}

export function MessageList({ messages, hasMore, loading, onLoadMore }: MessageListProps) {
  const scrollerRef = useRef<HTMLDivElement>(null);
  const loadingMoreRef = useRef(false);
  const shouldStickToBottomRef = useRef(true);

  const ordered = useMemo(() => [...messages].reverse(), [messages]);

  useEffect(() => {
    const scroller = scrollerRef.current;
    if (!scroller || !shouldStickToBottomRef.current) {
      return;
    }
    scroller.scrollTop = scroller.scrollHeight;
  }, [ordered.length]);

  async function handleScroll() {
    const scroller = scrollerRef.current;
    if (!scroller) return;

    const distanceToBottom = scroller.scrollHeight - scroller.scrollTop - scroller.clientHeight;
    shouldStickToBottomRef.current = distanceToBottom < 48;

    if (scroller.scrollTop > 20 || !hasMore || loadingMoreRef.current || loading) {
      return;
    }

    loadingMoreRef.current = true;
    const previousHeight = scroller.scrollHeight;
    await onLoadMore();

    requestAnimationFrame(() => {
      const next = scrollerRef.current;
      if (!next) return;
      next.scrollTop = next.scrollHeight - previousHeight;
      loadingMoreRef.current = false;
    });
  }

  return (
    <div
      ref={scrollerRef}
      onScroll={handleScroll}
      className="h-full overflow-y-auto px-4 py-3 space-y-3"
    >
      {hasMore && !loading && (
        <div className="text-center text-xs text-ctp-subtext0">Scroll up to load older messages</div>
      )}
      {loading && ordered.length === 0 ? (
        <div className="flex items-center justify-center p-6 text-ctp-subtext0">
          <div className="animate-pulse">Loading messages...</div>
        </div>
      ) : ordered.length === 0 ? (
        <div className="rounded-lg border border-dashed border-ctp-overlay0 p-6 text-center text-ctp-subtext0">
          No messages yet.
        </div>
      ) : (
        ordered.map((message) => <MessageItem key={message.id} message={message} />)
      )}
    </div>
  );
}
