import { create } from "zustand";

import {
  addAllowlist,
  approveBot,
  banMember,
  kickMember,
  listAllowlist,
  listBans,
  listMembers,
  listPendingBots,
  removeAllowlist,
  revokeBot,
  unbanMember,
  type AllowlistEntry,
  type Ban,
  type BotApproval,
  type Member,
} from "../api/members";

interface MembersStore {
  members: Member[];
  bans: Ban[];
  allowlist: AllowlistEntry[];
  pendingBots: BotApproval[];
  loading: boolean;
  error: string | null;
  fetchMembers: (baseUrl: string, token: string) => Promise<void>;
  fetchBans: (baseUrl: string, token: string) => Promise<void>;
  fetchAllowlist: (baseUrl: string, token: string) => Promise<void>;
  fetchPendingBots: (baseUrl: string, token: string) => Promise<void>;
  kick: (baseUrl: string, token: string, pubkey: string) => Promise<void>;
  ban: (baseUrl: string, token: string, pubkey: string, reason?: string) => Promise<void>;
  unban: (baseUrl: string, token: string, pubkey: string) => Promise<void>;
  addAllowlistEntry: (baseUrl: string, token: string, pubkey: string) => Promise<void>;
  removeAllowlistEntry: (baseUrl: string, token: string, pubkey: string) => Promise<void>;
  approveBotEntry: (baseUrl: string, token: string, pubkey: string, note?: string) => Promise<void>;
  revokeBotEntry: (baseUrl: string, token: string, pubkey: string) => Promise<void>;
  updateMemberProfile: (
    userId: string,
    patch: { display_name?: string | null; avatar_hash?: string | null },
  ) => void;
  applyGatewayEvent: (event: { op: string; d: unknown }) => void;
  clear: () => void;
}

export const useMembersStore = create<MembersStore>((set) => ({
  members: [],
  bans: [],
  allowlist: [],
  pendingBots: [],
  loading: false,
  error: null,

  fetchMembers: async (baseUrl, token) => {
    set({ loading: true, error: null });
    try {
      const members = await listMembers(baseUrl, token);
      set({ members, loading: false });
    } catch (error) {
      set({ loading: false, error: error instanceof Error ? error.message : String(error) });
    }
  },

  fetchBans: async (baseUrl, token) => {
    set({ loading: true, error: null });
    try {
      const bans = await listBans(baseUrl, token);
      set({ bans, loading: false });
    } catch (error) {
      set({ loading: false, error: error instanceof Error ? error.message : String(error) });
    }
  },

  fetchAllowlist: async (baseUrl, token) => {
    set({ loading: true, error: null });
    try {
      const allowlist = await listAllowlist(baseUrl, token);
      set({ allowlist, loading: false });
    } catch (error) {
      set({ loading: false, error: error instanceof Error ? error.message : String(error) });
    }
  },

  kick: async (baseUrl, token, pubkey) => {
    await kickMember(baseUrl, token, pubkey);
    set((state) => ({ members: state.members.filter((member) => member.pubkey !== pubkey) }));
  },

  ban: async (baseUrl, token, pubkey, reason) => {
    const created = await banMember(baseUrl, token, pubkey, reason);
    set((state) => ({
      bans: [created, ...state.bans.filter((ban) => ban.pubkey !== pubkey)],
      members: state.members.filter((member) => member.pubkey !== pubkey),
    }));
  },

  unban: async (baseUrl, token, pubkey) => {
    await unbanMember(baseUrl, token, pubkey);
    set((state) => ({ bans: state.bans.filter((ban) => ban.pubkey !== pubkey) }));
  },

  addAllowlistEntry: async (baseUrl, token, pubkey) => {
    const created = await addAllowlist(baseUrl, token, pubkey);
    set((state) => ({
      allowlist: [created, ...state.allowlist.filter((entry) => entry.pubkey !== pubkey)],
    }));
  },

  removeAllowlistEntry: async (baseUrl, token, pubkey) => {
    await removeAllowlist(baseUrl, token, pubkey);
    set((state) => ({ allowlist: state.allowlist.filter((entry) => entry.pubkey !== pubkey) }));
  },

  fetchPendingBots: async (baseUrl, token) => {
    set({ loading: true, error: null });
    try {
      const pendingBots = await listPendingBots(baseUrl, token);
      set({ pendingBots, loading: false });
    } catch (error) {
      set({ loading: false, error: error instanceof Error ? error.message : String(error) });
    }
  },

  approveBotEntry: async (baseUrl, token, pubkey, note) => {
    await approveBot(baseUrl, token, pubkey, note);
    set((state) => ({
      pendingBots: state.pendingBots.filter((bot) => bot.pubkey !== pubkey),
    }));
  },

  revokeBotEntry: async (baseUrl, token, pubkey) => {
    await revokeBot(baseUrl, token, pubkey);
  },

  updateMemberProfile: (userId, patch) => {
    set((state) => ({
      members: state.members.map((member) =>
        member.user_id === userId
          ? {
              ...member,
              ...(patch.display_name !== undefined ? { display_name: patch.display_name } : {}),
              ...(patch.avatar_hash !== undefined ? { avatar_hash: patch.avatar_hash } : {}),
            }
          : member,
      ),
    }));
  },

  applyGatewayEvent: (event) => {
    set((state) => {
      switch (event.op) {
        case "MEMBER_JOIN": {
          const payload = event.d as { user_id: string; pubkey: string; display_name?: string | null; avatar_hash?: string | null; joined_at: string; is_bot?: boolean; roles?: { id: string; name: string; color: string | null; position: number }[] };
          const existing = state.members.find((member) => member.user_id === payload.user_id);
          if (existing) {
            // Update profile fields in case they arrived incomplete in an earlier event.
            return {
              members: state.members.map((m) =>
                m.user_id === payload.user_id
                  ? {
                      ...m,
                      display_name: payload.display_name ?? m.display_name,
                      avatar_hash: payload.avatar_hash ?? m.avatar_hash,
                      is_bot: payload.is_bot ?? m.is_bot,
                    }
                  : m,
              ),
            };
          }
          return {
            members: [
              ...state.members,
              {
                user_id: payload.user_id,
                pubkey: payload.pubkey,
                display_name: payload.display_name ?? null,
                avatar_hash: payload.avatar_hash ?? null,
                joined_at: payload.joined_at,
                is_bot: payload.is_bot ?? false,
                roles: payload.roles ?? [],
              },
            ],
          };
        }
        case "MEMBER_LEAVE":
        case "MEMBER_KICK": {
          const payload = event.d as { user_id: string };
          return { members: state.members.filter((member) => member.user_id !== payload.user_id) };
        }
        case "MEMBER_BAN": {
          const payload = event.d as { pubkey: string };
          return { members: state.members.filter((member) => member.pubkey !== payload.pubkey) };
        }
        case "MEMBER_UPDATE": {
          const payload = event.d as {
            user_id: string;
            display_name?: string | null;
            avatar_hash?: string | null;
          };
          return {
            members: state.members.map((member) =>
              member.user_id === payload.user_id
                ? {
                    ...member,
                    ...(payload.display_name !== undefined
                      ? { display_name: payload.display_name }
                      : {}),
                    ...(payload.avatar_hash !== undefined
                      ? { avatar_hash: payload.avatar_hash }
                      : {}),
                  }
                : member,
            ),
          };
        }
        default:
          return {};
      }
    });
  },

  clear: () => {
    set({ members: [], bans: [], allowlist: [], pendingBots: [], loading: false, error: null });
  },
}));
