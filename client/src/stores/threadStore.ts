import { create } from "zustand";
import type { Message, Thread } from "../api/threads";
import { getThread, listThreadMessages, createThreadMessage, markThreadRead } from "../api/threads";
import { useServerStore, type GatewayEvent, type ReactionEvent } from "./serverStore";

interface ThreadState {
  // messages by threadId
  threadsMessages: Record<string, Message[]>;
  threads: Record<string, Thread>;
  isLoading: Record<string, boolean>;
  hasMore: Record<string, boolean>;
  unreadCounts: Record<string, number>;

  fetchThreadMessages: (threadId: string) => Promise<void>;
  sendMessage: (threadId: string, content: string, attachmentIds?: string[]) => Promise<void>;
  handleGatewayEvent: (event: GatewayEvent) => void;
  markAsRead: (threadId: string) => void;
}

export const useThreadStore = create<ThreadState>((set, get) => ({
  threadsMessages: {},
  threads: {},
  isLoading: {},
  hasMore: {},
  unreadCounts: {},

  fetchThreadMessages: async (threadId) => {
    if (get().threadsMessages[threadId] && !get().unreadCounts[threadId]) return;

    const { address, sessionToken } = useServerStore.getState();
    if (!address || !sessionToken) return;

    set((s) => ({ isLoading: { ...s.isLoading, [threadId]: true } }));

    try {
      const [thread, page] = await Promise.all([
        getThread(address, sessionToken, threadId),
        listThreadMessages(address, sessionToken, threadId, undefined, undefined, 50),
      ]);

      set((s) => ({
        threads: { ...s.threads, [threadId]: thread },
        threadsMessages: { ...s.threadsMessages, [threadId]: page.messages },
        hasMore: { ...s.hasMore, [threadId]: page.has_more },
        isLoading: { ...s.isLoading, [threadId]: false },
      }));

      get().markAsRead(threadId);
    } catch (error) {
      console.error("Failed to load thread:", error);
      set((s) => ({ isLoading: { ...s.isLoading, [threadId]: false } }));
    }
  },

  sendMessage: async (threadId, content, attachmentIds) => {
    const { address, sessionToken } = useServerStore.getState();
    if (!address || !sessionToken) return;

    await createThreadMessage(address, sessionToken, threadId, content, attachmentIds);
  },

  handleGatewayEvent: (event) => {
    const { sessionUserId } = useServerStore.getState();
    set((state) => {
      switch (event.op) {
        case "THREAD_MESSAGE_CREATE": {
          const { thread_id, message } = event.d as { thread_id: string; message: Message };
          const existing = state.threadsMessages[thread_id] ?? [];
          if (existing.some((m) => m.id === message.id)) return {};
          
          return {
            threadsMessages: {
              ...state.threadsMessages,
              [thread_id]: [...existing, message],
            },
          };
        }
        case "THREAD_UPDATE": {
          const { thread_id, reply_count, last_reply_at } = event.d as {
            thread_id: string;
            reply_count: number;
            last_reply_at: string | null;
          };
          if (state.threads[thread_id]) {
            return {
              threads: {
                ...state.threads,
                [thread_id]: {
                  ...state.threads[thread_id],
                  reply_count,
                  last_reply_at,
                },
              },
            };
          }
          return {};
        }
        case "REACTION_ADD": {
          const ev = event.d as ReactionEvent;
          const nextThreadsMessages = { ...state.threadsMessages };
          let changed = false;

          for (const [threadId, messages] of Object.entries(nextThreadsMessages)) {
            const updated = messages.map((m) => {
              if (m.id !== ev.message_id) return m;
              changed = true;
              const reactions = m.reactions.filter((r) => r.emoji !== ev.emoji);
              const prev = m.reactions.find((r) => r.emoji === ev.emoji);
              reactions.push({
                emoji: ev.emoji,
                count: (prev?.count ?? 0) + 1,
                me: ev.user_id === sessionUserId ? true : (prev?.me ?? false),
              });
              return { ...m, reactions };
            });
            if (changed) {
              nextThreadsMessages[threadId] = updated;
              break; // Optimization: message IDs are unique
            }
          }
          return changed ? { threadsMessages: nextThreadsMessages } : {};
        }
        case "REACTION_REMOVE": {
          const ev = event.d as ReactionEvent;
          const nextThreadsMessages = { ...state.threadsMessages };
          let changed = false;

          for (const [threadId, messages] of Object.entries(nextThreadsMessages)) {
            const updated = messages.map((m) => {
              if (m.id !== ev.message_id) return m;
              changed = true;
              const reactions = m.reactions
                .map((r) => {
                  if (r.emoji !== ev.emoji) return r;
                  return {
                    ...r,
                    count: r.count - 1,
                    me: ev.user_id === sessionUserId ? false : r.me,
                  };
                })
                .filter((r) => r.count > 0);
              return { ...m, reactions };
            });
            if (changed) {
              nextThreadsMessages[threadId] = updated;
              break;
            }
          }
          return changed ? { threadsMessages: nextThreadsMessages } : {};
        }
        default:
          return {};
      }
    });
  },

  markAsRead: (threadId) => {
    set((state) => {
      const next = { ...state.unreadCounts };
      delete next[threadId];
      return { unreadCounts: next };
    });

    const { address, sessionToken } = useServerStore.getState();
    if (address && sessionToken) {
      void markThreadRead(address, sessionToken, threadId);
    }
  },
}));
