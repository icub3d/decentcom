import { apiRequest } from "../services/api";

export interface Role {
  id: string;
  name: string;
  color: string | null;
  permissions: number;
  position: number;
  is_builtin: boolean;
  created_at: string;
  updated_at: string;
}

export interface ChannelPermissionOverride {
  channel_id: string;
  role_id: string;
  allow: number;
  deny: number;
}

export interface ListRolesResponse {
  roles: Role[];
}

export interface ListChannelOverridesResponse {
  overrides: ChannelPermissionOverride[];
}

export interface CreateRoleInput {
  name: string;
  color?: string | null;
  permissions: number;
  position?: number;
}

export interface UpdateRoleInput {
  name?: string;
  color?: string | null;
  permissions?: number;
  position?: number;
}

export async function listRoles(baseUrl: string, token: string): Promise<Role[]> {
  const response = await apiRequest<ListRolesResponse>(baseUrl, "/api/v1/roles", { token });
  return response.roles;
}

export async function createRole(
  baseUrl: string,
  token: string,
  input: CreateRoleInput,
): Promise<Role> {
  return apiRequest<Role>(baseUrl, "/api/v1/roles", {
    method: "POST",
    token,
    body: JSON.stringify(input),
  });
}

export async function updateRole(
  baseUrl: string,
  token: string,
  roleId: string,
  input: UpdateRoleInput,
): Promise<Role> {
  return apiRequest<Role>(baseUrl, `/api/v1/roles/${encodeURIComponent(roleId)}`, {
    method: "PATCH",
    token,
    body: JSON.stringify(input),
  });
}

export async function deleteRole(baseUrl: string, token: string, roleId: string): Promise<void> {
  await apiRequest<void>(baseUrl, `/api/v1/roles/${encodeURIComponent(roleId)}`, {
    method: "DELETE",
    token,
  });
}

export async function listChannelOverrides(
  baseUrl: string,
  token: string,
  channelId: string,
): Promise<ChannelPermissionOverride[]> {
  const response = await apiRequest<ListChannelOverridesResponse>(
    baseUrl,
    `/api/v1/channels/${encodeURIComponent(channelId)}/overrides`,
    { token },
  );
  return response.overrides;
}

export async function setChannelOverride(
  baseUrl: string,
  token: string,
  channelId: string,
  roleId: string,
  allow: number,
  deny: number,
): Promise<ChannelPermissionOverride> {
  return apiRequest<ChannelPermissionOverride>(
    baseUrl,
    `/api/v1/channels/${encodeURIComponent(channelId)}/overrides/${encodeURIComponent(roleId)}`,
    {
      method: "PUT",
      token,
      body: JSON.stringify({ allow, deny }),
    },
  );
}

export async function deleteChannelOverride(
  baseUrl: string,
  token: string,
  channelId: string,
  roleId: string,
): Promise<void> {
  await apiRequest<void>(
    baseUrl,
    `/api/v1/channels/${encodeURIComponent(channelId)}/overrides/${encodeURIComponent(roleId)}`,
    {
      method: "DELETE",
      token,
    },
  );
}
