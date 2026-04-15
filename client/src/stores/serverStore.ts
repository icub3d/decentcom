import { create } from "zustand";

import type { CreateCategoryRequest, CreateChannelRequest, UpdateCategoryRequest, UpdateChannelRequest } from "../api/channels";
import * as channelsApi from "../api/channels";
import type { ChannelPermissionOverride, Role } from "../api/roles";
import { listRoles } from "../api/roles";
import { ApiError, apiRequest } from "../services/api";
import { authenticateServer } from "../services/auth";
import { GatewayClient } from "../services/gateway";
import { useMembersStore } from "./members";
import { joinServer } from "../api/members";

export interface Channel {
  id: string;
  name: string;
  category_id: string | null;
  position: number;
  type: string;
}

export interface Category {
  id: string;
  name: string;
  position: number;
}

export interface Message {
  id: string;
  channel_id: string;
  author_id: string;
  content: string;
  created_at: string;
  edited_at: string | null;
  deleted: boolean;
}

interface ChannelsResponse {
  channels: Channel[];
  categories: Category[];
}

interface MessagePage {
  messages: Message[];
  has_more: boolean;
}

type ConnectionStatus = "connecting" | "connected" | "disconnected";

export interface GatewayEvent {
  op: string;
  d:
    | Message
    | Channel
    | Category
    | Role
    | { id: string; channel_id?: string }
    | { user_id: string; role_id: string }
    | { user_id: string; pubkey: string; roles: string[]; joined_at: string }
    | { user_id: string; pubkey: string; left_at: string }
    | { user_id: string; pubkey: string; kicked_by: string; kicked_at: string }
    | { pubkey: string; banned_by: string; reason?: string; banned_at: string }
    | { user_id: string; display_name?: string | null; avatar_hash?: string | null };
  t: number;
}

export interface ServerStore {
  serverId: string;
  address: string;
  status: ConnectionStatus;
  sessionToken: string | null;
  sessionUserId: string | null;
  channels: Channel[];
  categories: Category[];
  roles: Role[];
  memberRoleIdsByUserId: Record<string, string[]>;
  channelOverridesByChannelId: Record<string, ChannelPermissionOverride[]>;
  currentChannelId: string | null;
  messages: Record<string, Message[]>;
  hasMore: Record<string, boolean>;
  error: string | null;
  gateway: GatewayClient | null;

  connect: (address: string) => Promise<void>;
  disconnect: () => void;
  setCurrentChannel: (id: string) => Promise<void>;
  sendMessage: (content: string) => Promise<void>;
  loadMoreMessages: (channelId: string) => Promise<void>;
  createChannel: (req: CreateChannelRequest) => Promise<void>;
  updateChannel: (channelId: string, req: UpdateChannelRequest) => Promise<void>;
  deleteChannel: (channelId: string) => Promise<void>;
  createCategory: (req: CreateCategoryRequest) => Promise<Category>;
  updateCategory: (categoryId: string, req: UpdateCategoryRequest) => Promise<void>;
  deleteCategory: (categoryId: string) => Promise<void>;
  setStatus: (status: ConnectionStatus) => void;
  handleGatewayEvent: (event: GatewayEvent) => void;
}

function normalizeAddress(address: string): string {
  const trimmed = address.trim().replace(/\/$/, "");
  if (trimmed.startsWith("http://") || trimmed.startsWith("https://")) {
    return trimmed;
  }
  return `http://${trimmed}`;
}

function sortChannels(channels: Channel[]): Channel[] {
  return [...channels].sort((a, b) => a.position - b.position || a.name.localeCompare(b.name));
}

function sortCategories(categories: Category[]): Category[] {
  return [...categories].sort((a, b) => a.position - b.position || a.name.localeCompare(b.name));
}

export const useServerStore = create<ServerStore>((set, get) => ({
  serverId: "",
  address: "",
  status: "disconnected",
  sessionToken: null,
  sessionUserId: null,
  channels: [],
  categories: [],
  roles: [],
  memberRoleIdsByUserId: {},
  channelOverridesByChannelId: {},
  currentChannelId: null,
  messages: {},
  hasMore: {},
  error: null,
  gateway: null,

  connect: async (address: string) => {
    const normalized = normalizeAddress(address);
    set({ status: "connecting", error: null, address: normalized, serverId: normalized });

    try {
      const session = await authenticateServer(normalized);
      set({ sessionToken: session.token, sessionUserId: session.userId });

      let channelData;
      try {
        channelData = await apiRequest<ChannelsResponse>(normalized, "/api/v1/channels", {
          token: session.token,
        });
      } catch (error) {
        if (error instanceof ApiError && error.status === 403 && error.message.includes("membership required")) {
          // If we are not a member, try to join. This will only succeed if the server is in Open mode
          // or if the user is otherwise allowed to join (e.g. they joined before).
          await joinServer(normalized, session.token);
          // Retry channel fetch
          channelData = await apiRequest<ChannelsResponse>(normalized, "/api/v1/channels", {
            token: session.token,
          });
        } else {
          throw error;
        }
      }

      const roleData = await listRoles(normalized, session.token);
      await useMembersStore.getState().fetchMembers(normalized, session.token);

      const members = useMembersStore.getState().members;
      const roleMap: Record<string, string[]> = {};
      for (const member of members) {
        roleMap[member.user_id] = member.roles.map((r) => r.id);
      }

      const firstChannelId = channelData.channels[0]?.id ?? null;

      set({
        channels: sortChannels(channelData.channels),
        categories: sortCategories(channelData.categories),
        roles: roleData,
        memberRoleIdsByUserId: roleMap,
        currentChannelId: firstChannelId,
      });

      const gateway = get().gateway ?? new GatewayClient(get);
      set({ gateway });
      gateway.reconnect();

      if (firstChannelId) {
        await get().loadMoreMessages(firstChannelId);
      }

      set({ status: "connected" });
    } catch (error) {
      if (error instanceof ApiError && error.status === 403) {
        set({ status: "disconnected", error: error.message, sessionToken: null, sessionUserId: null });
        throw error;
      }
      const message = error instanceof Error ? error.message : String(error);
      set({ status: "disconnected", error: message, sessionToken: null, sessionUserId: null });
      throw error;
    }
  },

  disconnect: () => {
    get().gateway?.disconnect();
    useMembersStore.getState().clear();
    set({
      status: "disconnected",
      sessionToken: null,
      sessionUserId: null,
      channels: [],
      categories: [],
      roles: [],
      memberRoleIdsByUserId: {},
      channelOverridesByChannelId: {},
      currentChannelId: null,
      messages: {},
      hasMore: {},
    });
  },

  setCurrentChannel: async (id: string) => {
    const { currentChannelId, gateway } = get();
    if (currentChannelId === id) {
      return;
    }

    if (currentChannelId) {
      gateway?.unsubscribe(currentChannelId);
    }

    set({ currentChannelId: id });
    gateway?.subscribe(id);

    if (!get().messages[id]) {
      await get().loadMoreMessages(id);
    }
  },

  sendMessage: async (content: string) => {
    const state = get();
    if (!state.currentChannelId || !state.sessionToken) {
      return;
    }

    await apiRequest<Message>(
      state.address,
      `/api/v1/channels/${state.currentChannelId}/messages`,
      {
        method: "POST",
        token: state.sessionToken,
        body: JSON.stringify({ content }),
      },
    );
  },

  loadMoreMessages: async (channelId: string) => {
    const state = get();
    if (!state.sessionToken) {
      return;
    }

    const existing = state.messages[channelId] ?? [];
    const oldestId = existing.at(-1)?.id;
    const query = oldestId ? `?before=${encodeURIComponent(oldestId)}&limit=50` : "?limit=50";

    const page = await apiRequest<MessagePage>(
      state.address,
      `/api/v1/channels/${channelId}/messages${query}`,
      {
        token: state.sessionToken,
      },
    );

    set((prev) => ({
      messages: {
        ...prev.messages,
        [channelId]: [...existing, ...page.messages],
      },
      hasMore: {
        ...prev.hasMore,
        [channelId]: page.has_more,
      },
    }));
  },

  createChannel: async (req: CreateChannelRequest) => {
    const { address, sessionToken } = get();
    if (!sessionToken) return;
    await channelsApi.createChannel(address, sessionToken, req);
  },

  updateChannel: async (channelId: string, req: UpdateChannelRequest) => {
    const { address, sessionToken } = get();
    if (!sessionToken) return;
    await channelsApi.updateChannel(address, sessionToken, channelId, req);
  },

  deleteChannel: async (channelId: string) => {
    const { address, sessionToken } = get();
    if (!sessionToken) return;
    await channelsApi.deleteChannel(address, sessionToken, channelId);
  },

  createCategory: async (req: CreateCategoryRequest) => {
    const { address, sessionToken } = get();
    if (!sessionToken) throw new Error("Not connected");
    return channelsApi.createCategory(address, sessionToken, req);
  },

  updateCategory: async (categoryId: string, req: UpdateCategoryRequest) => {
    const { address, sessionToken } = get();
    if (!sessionToken) return;
    await channelsApi.updateCategory(address, sessionToken, categoryId, req);
  },

  deleteCategory: async (categoryId: string) => {
    const { address, sessionToken } = get();
    if (!sessionToken) return;
    await channelsApi.deleteCategory(address, sessionToken, categoryId);
  },

  setStatus: (status) => set({ status }),

  handleGatewayEvent: (event: GatewayEvent) => {
    set((state) => {
      switch (event.op) {
        case "MESSAGE_CREATE": {
          const message = event.d as Message;
          const existing = state.messages[message.channel_id] ?? [];
          if (existing.some((m) => m.id === message.id)) {
            return {};
          }
          return {
            messages: {
              ...state.messages,
              [message.channel_id]: [message, ...existing],
            },
          };
        }
        case "MESSAGE_UPDATE": {
          const message = event.d as Message;
          const existing = state.messages[message.channel_id] ?? [];
          return {
            messages: {
              ...state.messages,
              [message.channel_id]: existing.map((m) => (m.id === message.id ? message : m)),
            },
          };
        }
        case "MESSAGE_DELETE": {
          const deleted = event.d as { id: string; channel_id: string };
          const existing = state.messages[deleted.channel_id] ?? [];
          return {
            messages: {
              ...state.messages,
              [deleted.channel_id]: existing.map((m) =>
                m.id === deleted.id ? { ...m, deleted: true, content: "" } : m,
              ),
            },
          };
        }
        case "CHANNEL_CREATE": {
          const channel = event.d as Channel;
          return { channels: sortChannels([...state.channels, channel]) };
        }
        case "CHANNEL_UPDATE": {
          const channel = event.d as Channel;
          return {
            channels: sortChannels(
              state.channels.map((c) => (c.id === channel.id ? channel : c)),
            ),
          };
        }
        case "CHANNEL_DELETE": {
          const deleted = event.d as { id: string };
          const nextChannels = state.channels.filter((c) => c.id !== deleted.id);
          const nextCurrent =
            state.currentChannelId === deleted.id ? nextChannels[0]?.id ?? null : state.currentChannelId;
          return { channels: nextChannels, currentChannelId: nextCurrent };
        }
        case "CATEGORY_CREATE": {
          const category = event.d as Category;
          return { categories: sortCategories([...state.categories, category]) };
        }
        case "CATEGORY_UPDATE": {
          const category = event.d as Category;
          return {
            categories: sortCategories(
              state.categories.map((c) => (c.id === category.id ? category : c)),
            ),
          };
        }
        case "CATEGORY_DELETE": {
          const deleted = event.d as { id: string };
          return { categories: state.categories.filter((c) => c.id !== deleted.id) };
        }
        case "ROLE_CREATE": {
          const role = event.d as Role;
          return {
            roles: [...state.roles.filter((r) => r.id !== role.id), role].sort(
              (a, b) => b.position - a.position || a.name.localeCompare(b.name),
            ),
          };
        }
        case "ROLE_UPDATE": {
          const role = event.d as Role;
          return {
            roles: state.roles
              .map((r) => (r.id === role.id ? role : r))
              .sort((a, b) => b.position - a.position || a.name.localeCompare(b.name)),
          };
        }
        case "ROLE_DELETE": {
          const deleted = event.d as { id: string };
          const nextAssignments: Record<string, string[]> = {};
          for (const [userId, roleIds] of Object.entries(state.memberRoleIdsByUserId)) {
            nextAssignments[userId] = roleIds.filter((roleId) => roleId !== deleted.id);
          }
          return {
            roles: state.roles.filter((role) => role.id !== deleted.id),
            memberRoleIdsByUserId: nextAssignments,
          };
        }
        case "MEMBER_ROLE_ADD": {
          const payload = event.d as { user_id: string; role_id: string };
          const existing = state.memberRoleIdsByUserId[payload.user_id] ?? [];
          if (existing.includes(payload.role_id)) {
            return {};
          }
          return {
            memberRoleIdsByUserId: {
              ...state.memberRoleIdsByUserId,
              [payload.user_id]: [...existing, payload.role_id],
            },
          };
        }
        case "MEMBER_ROLE_REMOVE": {
          const payload = event.d as { user_id: string; role_id: string };
          const existing = state.memberRoleIdsByUserId[payload.user_id] ?? [];
          return {
            memberRoleIdsByUserId: {
              ...state.memberRoleIdsByUserId,
              [payload.user_id]: existing.filter((roleId) => roleId !== payload.role_id),
            },
          };
        }
        case "MEMBER_JOIN": {
          const member = event.d as { user_id: string; pubkey: string; roles: string[]; joined_at: string };
          const existing = state.memberRoleIdsByUserId[member.user_id] ?? [];
          const merged = Array.from(new Set([...existing, ...member.roles]));
          useMembersStore.getState().applyGatewayEvent(event);
          return {
            memberRoleIdsByUserId: {
              ...state.memberRoleIdsByUserId,
              [member.user_id]: merged,
            },
          };
        }
        case "MEMBER_LEAVE":
        case "MEMBER_KICK": {
          const payload = event.d as { user_id: string };
          useMembersStore.getState().applyGatewayEvent(event);
          const next = { ...state.memberRoleIdsByUserId };
          delete next[payload.user_id];
          return { memberRoleIdsByUserId: next };
        }
        case "MEMBER_BAN": {
          useMembersStore.getState().applyGatewayEvent(event);
          return {};
        }
        case "MEMBER_UPDATE": {
          useMembersStore.getState().applyGatewayEvent(event);
          return {};
        }
        default:
          return {};
      }
    });
  },
}));
