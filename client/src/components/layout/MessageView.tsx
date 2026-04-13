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
  if (!channel) {
    return (
      <section className="flex-1 flex items-center justify-center bg-slate-900 text-slate-500">
        Select a channel to begin chatting.
      </section>
    );
  }

  return (
    <section className="flex h-full flex-1 flex-col bg-slate-900">
      <header className="border-b border-slate-800 px-5 py-4">
        <h2 className="text-lg font-bold text-slate-100">#{channel.name}</h2>
      </header>
      <div className="flex-1 min-h-0">
        <MessageList messages={messages} hasMore={hasMore} onLoadMore={onLoadMore} />
      </div>
      <MessageInput disabled={!connected} onSend={onSend} />
    </section>
  );
}
