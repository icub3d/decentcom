import { useCallback, useEffect, useMemo, useState } from "react";

import { assignRole, removeRole } from "../../api/members";
import { useAppStore } from "../../stores/appStore";
import { useMembersStore } from "../../stores/members";
import { useServerStore } from "../../stores/serverStore";
import { MemberList } from "../members/MemberList";
import { ChannelSidebar } from "./ChannelSidebar";
import { MessageView } from "./MessageView";
import { ServerSidebar } from "./ServerSidebar";

export interface AppShellProps {
  onSwitchAccount: (pubkey: string) => void;
  onAddAccount: () => void;
}

export function AppShell({ onSwitchAccount, onAddAccount }: AppShellProps) {
  const { currentServerId, servers, setCurrentServer } = useAppStore();
  const {
    address,
    sessionToken,
    channels,
    currentChannelId,
    messages,
    hasMore,
    status,
    roles,
    memberRoleIdsByUserId,
    setCurrentChannel,
    sendMessage,
    loadMoreMessages,
  } = useServerStore();

  const members = useMembersStore((state) => state.members);
  const fetchMembers = useMembersStore((state) => state.fetchMembers);
  const kickMember = useMembersStore((state) => state.kick);
  const banMember = useMembersStore((state) => state.ban);

  const [memberPanelOpen, setMemberPanelOpen] = useState(true);

  // Fetch members when connected to a server
  useEffect(() => {
    if (address && sessionToken && status === "connected") {
      void fetchMembers(address, sessionToken);
    }
  }, [address, sessionToken, status, fetchMembers]);

  const handleKick = useCallback(
    async (pubkey: string) => {
      if (!address || !sessionToken) throw new Error("Not connected");
      await kickMember(address, sessionToken, pubkey);
    },
    [address, sessionToken, kickMember],
  );

  const handleBan = useCallback(
    async (pubkey: string, reason?: string) => {
      if (!address || !sessionToken) throw new Error("Not connected");
      await banMember(address, sessionToken, pubkey, reason);
    },
    [address, sessionToken, banMember],
  );

  const handleAssignRole = useCallback(
    async (pubkey: string, roleId: string) => {
      if (!address || !sessionToken) throw new Error("Not connected");
      await assignRole(address, sessionToken, pubkey, roleId);
    },
    [address, sessionToken],
  );

  const handleRemoveRole = useCallback(
    async (pubkey: string, roleId: string) => {
      if (!address || !sessionToken) throw new Error("Not connected");
      await removeRole(address, sessionToken, pubkey, roleId);
    },
    [address, sessionToken],
  );

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
        onSwitchAccount={onSwitchAccount}
        onAddAccount={onAddAccount}
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
        memberPanelOpen={memberPanelOpen}
        onToggleMemberPanel={() => setMemberPanelOpen((v) => !v)}
      />
      {memberPanelOpen && currentServerId && status === "connected" && (
        <aside className="w-64 border-l border-ctp-overlay0 bg-ctp-mantle overflow-y-auto shrink-0">
          <MemberList
            members={members}
            roles={roles}
            memberRoleIdsByUserId={memberRoleIdsByUserId}
            onKick={handleKick}
            onBan={handleBan}
            onAssignRole={handleAssignRole}
            onRemoveRole={handleRemoveRole}
          />
        </aside>
      )}
    </main>
  );
}
