import { create } from "zustand";

import {
  addAllowlist,
  banMember,
  kickMember,
  listAllowlist,
  listBans,
  listMembers,
  removeAllowlist,
  unbanMember,
  type AllowlistEntry,
  type Ban,
  type Member,
} from "../api/members";

interface MembersStore {
  members: Member[];
  bans: Ban[];
  allowlist: AllowlistEntry[];
  loading: boolean;
  error: string | null;
  fetchMembers: (baseUrl: string, token: string) => Promise<void>;
  fetchBans: (baseUrl: string, token: string) => Promise<void>;
  fetchAllowlist: (baseUrl: string, token: string) => Promise<void>;
  kick: (baseUrl: string, token: string, pubkey: string) => Promise<void>;
  ban: (baseUrl: string, token: string, pubkey: string, reason?: string) => Promise<void>;
  unban: (baseUrl: string, token: string, pubkey: string) => Promise<void>;
  addAllowlistEntry: (baseUrl: string, token: string, pubkey: string) => Promise<void>;
  removeAllowlistEntry: (baseUrl: string, token: string, pubkey: string) => Promise<void>;
  applyGatewayEvent: (event: { op: string; d: unknown }) => void;
  clear: () => void;
}

export const useMembersStore = create<MembersStore>((set) => ({
  members: [],
  bans: [],
  allowlist: [],
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

  applyGatewayEvent: (event) => {
    set((state) => {
      switch (event.op) {
        case "MEMBER_JOIN": {
          const payload = event.d as { user_id: string; pubkey: string; joined_at: string; roles?: { id: string; name: string; color: string | null; position: number }[] };
          const existing = state.members.find((member) => member.user_id === payload.user_id);
          if (existing) {
            return {};
          }
          return {
            members: [
              ...state.members,
              {
                user_id: payload.user_id,
                pubkey: payload.pubkey,
                joined_at: payload.joined_at,
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
        default:
          return {};
      }
    });
  },

  clear: () => {
    set({ members: [], bans: [], allowlist: [], loading: false, error: null });
  },
}));
