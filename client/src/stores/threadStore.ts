import { create } from "zustand";
import type { Message, Thread } from "../api/threads";
import { getThread, listThreadMessages, createThreadMessage, followThread, unfollowThread, markThreadRead } from "../api/threads";
import { useServerStore, type GatewayEvent } from "./serverStore";

interface ThreadState {
  activeThreadId: string | null;
  activeThread: Thread | null;
  messages: Message[];
  hasMore: boolean;
  isLoading: boolean;
  unreadCounts: Record<string, number>;

  setActiveThread: (threadId: string | null) => Promise<void>;
  loadMoreMessages: () => Promise<void>;
  sendMessage: (content: string, attachmentIds?: string[]) => Promise<void>;
  toggleFollow: () => Promise<void>;
  handleGatewayEvent: (event: GatewayEvent) => void;
  markAsRead: (threadId: string) => void;
}

export const useThreadStore = create<ThreadState>((set, get) => ({
  activeThreadId: null,
  activeThread: null,
  messages: [],
  hasMore: false,
  isLoading: false,
  unreadCounts: {},

  setActiveThread: async (threadId) => {
    if (get().activeThreadId === threadId) return;

    if (!threadId) {
      set({ activeThreadId: null, activeThread: null, messages: [], hasMore: false });
      return;
    }

    const { address, sessionToken } = useServerStore.getState();
    if (!address || !sessionToken) return;

    set({ activeThreadId: threadId, isLoading: true, messages: [] });

    try {
      const [thread, page] = await Promise.all([
        getThread(address, sessionToken, threadId),
        listThreadMessages(address, sessionToken, threadId, undefined, undefined, 50)
      ]);

      set({
        activeThread: thread,
        messages: page.messages,
        hasMore: page.has_more,
        isLoading: false
      });
      
      get().markAsRead(threadId);
    } catch (error) {
      console.error("Failed to load thread:", error);
      set({ isLoading: false });
    }
  },

  loadMoreMessages: async () => {
    const { activeThreadId, messages, hasMore, isLoading } = get();
    if (!activeThreadId || !hasMore || isLoading) return;

    const { address, sessionToken } = useServerStore.getState();
    if (!address || !sessionToken) return;

    const oldestId = messages[0]?.id;
    
    set({ isLoading: true });
    try {
      const page = await listThreadMessages(address, sessionToken, activeThreadId, oldestId, undefined, 50);
      set((state) => ({
        messages: [...page.messages, ...state.messages],
        hasMore: page.has_more,
        isLoading: false
      }));
    } catch (error) {
      console.error("Failed to load more thread messages:", error);
      set({ isLoading: false });
    }
  },

  sendMessage: async (content, attachmentIds) => {
    const { activeThreadId } = get();
    const { address, sessionToken } = useServerStore.getState();
    if (!activeThreadId || !address || !sessionToken) return;

    await createThreadMessage(address, sessionToken, activeThreadId, content, attachmentIds);
  },

  toggleFollow: async () => {
    const { activeThread, activeThreadId } = get();
    const { address, sessionToken } = useServerStore.getState();
    if (!activeThread || !activeThreadId || !address || !sessionToken) return;

    if (activeThread.is_following) {
      await unfollowThread(address, sessionToken, activeThreadId);
      set({ activeThread: { ...activeThread, is_following: false } });
    } else {
      await followThread(address, sessionToken, activeThreadId);
      set({ activeThread: { ...activeThread, is_following: true } });
    }
  },

  handleGatewayEvent: (event) => {
    const { activeThreadId, activeThread } = get();

    switch (event.op) {
      case "THREAD_MESSAGE_CREATE": {
        const { thread_id, message } = event.d as { thread_id: string; message: Message };
        if (activeThreadId === thread_id) {
          set((state) => ({
            messages: [...state.messages, message]
          }));
        } else {
          set((state) => ({
            unreadCounts: {
              ...state.unreadCounts,
              [thread_id]: (state.unreadCounts[thread_id] ?? 0) + 1
            }
          }));
        }
        break;
      }
      case "THREAD_UPDATE": {
        const { thread_id, reply_count, last_reply_at } = event.d as {
          thread_id: string;
          reply_count: number;
          last_reply_at: string | null;
        };
        if (activeThreadId === thread_id && activeThread) {
          set({
            activeThread: {
              ...activeThread,
              reply_count,
              last_reply_at
            }
          });
        }
        break;
      }
    }
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
  }
}));
