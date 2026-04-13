import { SEND_MESSAGES, usePermissions } from "../../hooks/usePermissions";
import type { Channel, Message } from "../../stores/serverStore";
import { MessageList } from "../messages/MessageList";
import { MessageInput } from "./MessageInput";

interface MessageViewProps {
  channel: Channel | null;
  messages: Message[];
  hasMore: boolean;
  connected: boolean;
  onLoadMore: () => Promise<void>;
  onSend: (content: string) => Promise<void>;
}

export function MessageView({
  channel,
  messages,
  hasMore,
  connected,
  onLoadMore,
  onSend,
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
    <section className="flex h-full flex-1 flex-col bg-ctp-base">
      <header className="border-b border-ctp-overlay0 px-5 py-4">
        <h2 className="text-lg font-bold text-ctp-text">#{channel.name}</h2>
      </header>
      <div className="flex-1 min-h-0">
        <MessageList messages={messages} hasMore={hasMore} onLoadMore={onLoadMore} />
      </div>
      <MessageInput disabled={!connected || !permissions.has(SEND_MESSAGES)} onSend={onSend} />
    </section>
  );
}
