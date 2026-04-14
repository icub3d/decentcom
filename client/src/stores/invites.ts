import { create } from "zustand";

import {
  createInvite,
  getInvitePreview,
  joinInvite,
  listInvites,
  revokeInvite,
  type CreateInviteInput,
  type Invite,
  type InvitePreview,
  type JoinInviteResponse,
} from "../api/invites";

interface InvitesStore {
  invites: Invite[];
  preview: InvitePreview | null;
  loading: boolean;
  error: string | null;
  listInvites: (baseUrl: string, token: string) => Promise<void>;
  createInvite: (baseUrl: string, token: string, input: CreateInviteInput) => Promise<Invite>;
  revokeInvite: (baseUrl: string, token: string, code: string) => Promise<void>;
  getInvitePreview: (baseUrl: string, code: string) => Promise<InvitePreview>;
  joinInvite: (baseUrl: string, token: string, code: string) => Promise<JoinInviteResponse>;
  clearPreview: () => void;
}

export const useInvitesStore = create<InvitesStore>((set, get) => ({
  invites: [],
  preview: null,
  loading: false,
  error: null,

  listInvites: async (baseUrl, token) => {
    set({ loading: true, error: null });
    try {
      const invites = await listInvites(baseUrl, token);
      set({ invites, loading: false });
    } catch (error) {
      set({
        loading: false,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  },

  createInvite: async (baseUrl, token, input) => {
    const created = await createInvite(baseUrl, token, input);
    set((state) => ({
      invites: [created, ...state.invites.filter((invite) => invite.code !== created.code)],
    }));
    return created;
  },

  revokeInvite: async (baseUrl, token, code) => {
    await revokeInvite(baseUrl, token, code);
    set((state) => ({
      invites: state.invites.filter((invite) => invite.code !== code),
    }));
  },

  getInvitePreview: async (baseUrl, code) => {
    const preview = await getInvitePreview(baseUrl, code);
    set({ preview, error: null });
    return preview;
  },

  joinInvite: async (baseUrl, token, code) => {
    const response = await joinInvite(baseUrl, token, code);
    const invites = get().invites.map((invite) =>
      invite.code === code
        ? { ...invite, use_count: invite.use_count + 1 }
        : invite,
    );
    set({ invites });
    return response;
  },

  clearPreview: () => set({ preview: null }),
}));
