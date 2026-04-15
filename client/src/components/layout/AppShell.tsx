import { useMemo } from "react";

import { useAppStore } from "../../stores/appStore";
import { useServerStore } from "../../stores/serverStore";
import { ChannelSidebar } from "./ChannelSidebar";
import { MessageView } from "./MessageView";
import { ServerSidebar } from "./ServerSidebar";

export function AppShell() {
  const { currentServerId, servers, setCurrentServer } = useAppStore();
  const {
    channels,
    currentChannelId,
    messages,
    hasMore,
    status,
    setCurrentChannel,
    sendMessage,
    loadMoreMessages,
  } = useServerStore();

  const serverList = useMemo(() => Object.values(servers), [servers]);
  const currentChannel = channels.find((ch) => ch.id === currentChannelId) ?? null;
  const channelMessages = currentChannelId ? messages[currentChannelId] ?? [] : [];
  const channelHasMore = currentChannelId ? hasMore[currentChannelId] ?? false : false;

  return (
    <main className="flex-1 h-full bg-ctp-crust text-ctp-text flex overflow-hidden">
      <ServerSidebar
        servers={serverList}
        currentServerId={currentServerId}
        onSelectServer={setCurrentServer}
      />
      <ChannelSidebar
        channels={channels}
        currentChannelId={currentChannelId}
        status={status}
        onSelectChannel={(id) => {
          void setCurrentChannel(id);
        }}
      />
      <MessageView
        channel={currentChannel}
        messages={channelMessages}
        hasMore={channelHasMore}
        connected={status === "connected"}
        onLoadMore={() =>
          currentChannelId ? loadMoreMessages(currentChannelId) : Promise.resolve()
        }
        onSend={sendMessage}
      />
    </main>
  );
}
