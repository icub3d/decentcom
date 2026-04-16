import type { Message } from "../../stores/serverStore";
import { useMembersStore } from "../../stores/members";
import { useServerStore } from "../../stores/serverStore";
import { useThreadStore } from "../../stores/threadStore";
import { createThread } from "../../api/threads";
import { Avatar } from "../profile/Avatar";
import { MessageAttachment } from "./MessageAttachment";
import { ReactionBar } from "./ReactionBar";
import { ThreadIndicator } from "./ThreadIndicator";

interface MessageItemProps {
  message: Message;
}

function formatTime(timestamp: string): string {
  return new Date(timestamp).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  });
}

function truncatePubkey(pubkey: string): string {
  if (pubkey.length <= 14) {
    return pubkey;
  }
  return `${pubkey.slice(0, 8)}...${pubkey.slice(-6)}`;
}

export function MessageItem({ message }: MessageItemProps) {
  const member = useMembersStore((s) =>
    s.members.find((m) => m.user_id === message.author_id),
  );
  const { address, sessionToken } = useServerStore();
  const setActiveThread = useThreadStore((s) => s.setActiveThread);

  const displayName = member?.display_name ?? (member ? truncatePubkey(member.pubkey) : message.author_id);
  const pubkey = member?.pubkey ?? message.author_id;

  async function handleReplyInThread() {
    if (!address || !sessionToken) return;
    if (message.thread) {
      await setActiveThread(message.thread.thread_id);
    } else {
      try {
        const resp = await createThread(address, sessionToken, message.channel_id, message.id);
        await setActiveThread(resp.thread_id);
      } catch (error) {
        console.error("Failed to create thread:", error);
      }
    }
  }

  return (
    <article className="group relative flex gap-3 rounded-lg border border-ctp-overlay0 bg-ctp-mantle/60 px-4 py-3">
      <div className="mt-0.5 flex-shrink-0">
        <Avatar pubkey={pubkey} avatarHash={member?.avatar_hash} size={32} />
      </div>
      <div className="min-w-0 flex-1">
        <div className="mb-1 flex items-center justify-between">
          <div className="flex items-center gap-2 text-xs text-ctp-subtext0">
            <span className="font-semibold text-ctp-subtext1">{displayName}</span>
            <time>{formatTime(message.created_at)}</time>
            {message.edited_at && !message.deleted && <span className="text-ctp-yellow">(edited)</span>}
          </div>
          
          {!message.deleted && (
            <button
              onClick={() => void handleReplyInThread()}
              className="hidden group-hover:block text-[10px] font-bold text-ctp-subtext0 hover:text-ctp-blue transition-colors uppercase tracking-tight"
            >
              Reply in Thread
            </button>
          )}
        </div>
        
        {message.deleted ? (
          <p className="italic text-ctp-overlay1">This message was deleted.</p>
        ) : (
          <>
            {message.content && (
              <p className="whitespace-pre-wrap text-ctp-text">{message.content}</p>
            )}
            {message.attachments?.length > 0 && (
              <div className="flex flex-col gap-1">
                {message.attachments.map((a) => (
                  <MessageAttachment key={a.id} attachment={a} />
                ))}
              </div>
            )}
          </>
        )}
        
        <ReactionBar
          channelId={message.channel_id}
          messageId={message.id}
          reactions={message.reactions ?? []}
        />

        {message.thread && <ThreadIndicator summary={message.thread} />}
      </div>
    </article>
  );
}
