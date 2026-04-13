import { create } from "zustand";

import {
  createRole,
  deleteRole,
  listChannelOverrides,
  listRoles,
  setChannelOverride,
  type ChannelPermissionOverride,
  type CreateRoleInput,
  type Role,
  type UpdateRoleInput,
  updateRole,
} from "../api/roles";

interface RolesStore {
  roles: Role[];
  channelOverrides: Record<string, ChannelPermissionOverride[]>;
  loading: boolean;
  error: string | null;
  loadRoles: (baseUrl: string, token: string) => Promise<void>;
  createRole: (baseUrl: string, token: string, input: CreateRoleInput) => Promise<Role>;
  updateRole: (
    baseUrl: string,
    token: string,
    roleId: string,
    input: UpdateRoleInput,
  ) => Promise<Role>;
  deleteRole: (baseUrl: string, token: string, roleId: string) => Promise<void>;
  loadChannelOverrides: (baseUrl: string, token: string, channelId: string) => Promise<void>;
  setChannelOverride: (
    baseUrl: string,
    token: string,
    channelId: string,
    roleId: string,
    allow: number,
    deny: number,
  ) => Promise<void>;
}

function sortRoles(roles: Role[]): Role[] {
  return [...roles].sort((a, b) => b.position - a.position || a.name.localeCompare(b.name));
}

export const useRolesStore = create<RolesStore>((set, get) => ({
  roles: [],
  channelOverrides: {},
  loading: false,
  error: null,

  loadRoles: async (baseUrl, token) => {
    set({ loading: true, error: null });
    try {
      const roles = await listRoles(baseUrl, token);
      set({ roles: sortRoles(roles), loading: false });
    } catch (error) {
      set({
        loading: false,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  },

  createRole: async (baseUrl, token, input) => {
    const role = await createRole(baseUrl, token, input);
    set((state) => ({
      roles: sortRoles([...state.roles.filter((current) => current.id !== role.id), role]),
    }));
    return role;
  },

  updateRole: async (baseUrl, token, roleId, input) => {
    const role = await updateRole(baseUrl, token, roleId, input);
    set((state) => ({
      roles: sortRoles(state.roles.map((current) => (current.id === role.id ? role : current))),
    }));
    return role;
  },

  deleteRole: async (baseUrl, token, roleId) => {
    await deleteRole(baseUrl, token, roleId);
    set((state) => ({
      roles: state.roles.filter((role) => role.id !== roleId),
    }));
  },

  loadChannelOverrides: async (baseUrl, token, channelId) => {
    const overrides = await listChannelOverrides(baseUrl, token, channelId);
    set((state) => ({
      channelOverrides: {
        ...state.channelOverrides,
        [channelId]: overrides,
      },
    }));
  },

  setChannelOverride: async (baseUrl, token, channelId, roleId, allow, deny) => {
    const overrideRow = await setChannelOverride(baseUrl, token, channelId, roleId, allow, deny);
    const current = get().channelOverrides[channelId] ?? [];
    const next = [...current.filter((row) => row.role_id !== roleId), overrideRow];
    set((state) => ({
      channelOverrides: {
        ...state.channelOverrides,
        [channelId]: next,
      },
    }));
  },
}));
