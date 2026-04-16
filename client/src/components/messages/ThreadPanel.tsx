import { useEffect, useRef } from "react";
import { useThreadStore } from "../../stores/threadStore";
import { useServerStore } from "../../stores/serverStore";
import { MessageItem } from "./MessageItem";
import { MessageInput } from "../layout/MessageInput";

export function ThreadPanel() {
  const { activeThreadId, activeThread, messages, isLoading, setActiveThread, sendMessage, toggleFollow } = useThreadStore();
  const currentChannelId = useServerStore((s) => s.currentChannelId);
  const serverMessages = useServerStore((s) => s.messages);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages.length]);

  if (!activeThreadId) return null;

  // Find parent message in serverStore if not already in thread messages
  const parentMessage = currentChannelId ? serverMessages[currentChannelId]?.find((m) => m.id === activeThread?.parent_message_id) : null;

  return (
    <div className="flex h-full w-[400px] min-w-[300px] flex-col border-l border-ctp-surface1 bg-ctp-crust">
      <header className="flex h-12 items-center justify-between border-b border-ctp-surface1 px-4">
        <div className="flex items-center gap-2 font-semibold text-ctp-text">
          <span>💬 Thread</span>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => void toggleFollow()}
            className={`text-xs font-medium ${
              activeThread?.is_following ? "text-ctp-blue" : "text-ctp-subtext0"
            } hover:underline`}
          >
            {activeThread?.is_following ? "Following" : "Follow"}
          </button>
          <button
            onClick={() => void setActiveThread(null)}
            className="text-ctp-subtext0 hover:text-ctp-text"
          >
            ✕
          </button>
        </div>
      </header>

      <div ref={scrollRef} className="flex-1 overflow-y-auto p-4 space-y-4">
        {parentMessage && (
          <div className="pb-4 border-b border-ctp-surface1">
            <MessageItem message={parentMessage} />
            <div className="mt-2 text-xs font-medium text-ctp-subtext1 uppercase tracking-wider">
              {activeThread?.reply_count ?? 0} {activeThread?.reply_count === 1 ? "Reply" : "Replies"}
            </div>
          </div>
        )}

        {messages.map((m) => (
          <MessageItem key={m.id} message={m} />
        ))}
        
        {isLoading && (
          <div className="flex justify-center py-2 text-ctp-subtext0">
            Loading...
          </div>
        )}
      </div>

      <div className="p-4 border-t border-ctp-surface1 bg-ctp-mantle">
        <MessageInput 
          onSend={(content, attachments) => void sendMessage(content, attachments)}
          placeholder="Reply in thread..."
        />
      </div>
    </div>
  );
}
