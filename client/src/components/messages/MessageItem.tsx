import { useEffect } from "react";
import type { Message } from "../../stores/serverStore";
import { useMembersStore } from "../../stores/members";
import { useServerStore } from "../../stores/serverStore";
import { useThreadStore } from "../../stores/threadStore";
import { Avatar } from "../profile/Avatar";
import { MessageAttachment } from "./MessageAttachment";
import { ReactionBar } from "./ReactionBar";

interface MessageItemProps {
  message: Message;
  isReply?: boolean;
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

export function MessageItem({ message, isReply }: MessageItemProps) {
  const member = useMembersStore((s) =>
    s.members.find((m) => m.user_id === message.author_id),
  );
  const setReplyingTo = useServerStore((s) => s.setReplyingTo);
  const { threadsMessages, fetchThreadMessages, isLoading } = useThreadStore();
  
  const threadId = message.thread?.thread_id;
  const replies = threadId ? threadsMessages[threadId] ?? [] : [];

  useEffect(() => {
    if (threadId && message.thread && message.thread.reply_count > 0) {
      void fetchThreadMessages(threadId);
    }
  }, [threadId, message.thread, fetchThreadMessages]);

  const displayName = member?.display_name ?? (member ? truncatePubkey(member.pubkey) : message.author_id);
  const pubkey = member?.pubkey ?? message.author_id;

  return (
    <div className={`flex flex-col gap-1 ${isReply ? "ml-8" : ""}`}>
      <article className={`group relative flex gap-3 rounded-lg border border-ctp-overlay0 bg-ctp-mantle/60 px-4 py-3 ${isReply ? "scale-95 origin-left" : ""}`}>
        <div className="mt-0.5 flex-shrink-0">
          <Avatar pubkey={pubkey} avatarHash={member?.avatar_hash} size={isReply ? 24 : 32} />
        </div>
        <div className="min-w-0 flex-1">
          <div className="mb-1 flex items-center justify-between">
            <div className="flex items-center gap-2 text-xs text-ctp-subtext0">
              <span className="font-semibold text-ctp-subtext1">{displayName}</span>
              {member?.is_bot && (
                <span className="rounded bg-ctp-blue/20 px-1 py-0.5 text-[10px] font-bold text-ctp-blue">BOT</span>
              )}
              <time>{formatTime(message.created_at)}</time>
              {message.edited_at && !message.deleted && <span className="text-ctp-yellow">(edited)</span>}
            </div>
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
          
          <div className="mt-1.5 flex items-center justify-between gap-1">
            <ReactionBar
              channelId={message.channel_id}
              messageId={message.id}
              reactions={message.reactions ?? []}
            />

            {!message.deleted && !isReply && (
              <button
                onClick={() => setReplyingTo(message)}
                title="Reply"
                className="hidden group-hover:flex h-6 w-6 items-center justify-center rounded-md text-ctp-subtext0 hover:bg-ctp-surface0 hover:text-ctp-blue transition-all"
              >
                <span className="text-xl leading-[0] mb-1">↵</span>
              </button>
            )}
          </div>
        </div>
      </article>

      {/* Inline Replies */}
      {replies.length > 0 && (
        <div className="mt-1 flex flex-col gap-1 border-l-2 border-ctp-surface1 ml-4 pl-4">
          {replies.map((r) => (
            <MessageItem key={r.id} message={r} isReply />
          ))}
          {threadId && isLoading[threadId] && (
            <div className="text-[10px] text-ctp-subtext0 animate-pulse ml-8">Loading replies...</div>
          )}
        </div>
      )}
    </div>
  );
}
