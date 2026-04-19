import { SEND_MESSAGES, usePermissions } from "../../hooks/usePermissions";
import type { Channel, Message } from "../../stores/serverStore";
import { MessageList } from "../messages/MessageList";
import { MessageInput } from "./MessageInput";

interface MessageViewProps {
  channel: Channel | null;
  messages: Message[];
  hasMore: boolean;
  loading: boolean;
  connected: boolean;
  onLoadMore: () => Promise<void>;
  onSend: (content: string, attachmentIds?: string[]) => Promise<void>;
  memberPanelOpen?: boolean;
  onToggleMemberPanel?: () => void;
}

export function MessageView({
  channel,
  messages,
  hasMore,
  loading,
  connected,
  onLoadMore,
  onSend,
  memberPanelOpen,
  onToggleMemberPanel,
}: MessageViewProps) {
  const permissions = usePermissions(channel?.id);

  if (!channel) {
    return (
      <section className="flex-1 flex items-center justify-center bg-ctp-base text-ctp-subtext0">
        Select a channel to begin chatting.
      </section>
    );
  }

  return (
    <div className="flex h-full flex-1 overflow-hidden">
      <section className="flex h-full flex-1 flex-col bg-ctp-base min-w-0">
        <header className="border-b border-ctp-overlay0 px-5 py-4 flex items-center justify-between">
          <h2 className="text-lg font-bold text-ctp-text">#{channel.name}</h2>
          {onToggleMemberPanel && (
            <button
              onClick={onToggleMemberPanel}
              title={memberPanelOpen ? "Hide members" : "Show members"}
              className={`h-8 w-8 rounded-lg transition flex items-center justify-center text-sm ${
                memberPanelOpen
                  ? "bg-ctp-blue text-ctp-crust"
                  : "bg-ctp-surface0 text-ctp-subtext1 hover:bg-ctp-surface1"
              }`}
            >
              👥
            </button>
          )}
        </header>
        <div className="flex-1 min-h-0">
          <MessageList messages={messages} hasMore={hasMore} loading={loading} onLoadMore={onLoadMore} />
        </div>
        <MessageInput disabled={!connected || !permissions.has(SEND_MESSAGES)} onSend={onSend} />
      </section>
    </div>
  );
}
