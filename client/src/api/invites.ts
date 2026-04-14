import { apiRequest } from "../services/api";

export interface Invite {
  code: string;
  created_by: string;
  grant_role_id: string | null;
  max_uses: number;
  use_count: number;
  expires_at: string | null;
  created_at: string;
  invite_link: string;
}

export interface InvitePreview {
  code: string;
  server_name: string;
  server_icon: string | null;
  member_count: number;
  expires_at: string | null;
}

export interface JoinInviteResponse {
  member: {
    pubkey: string;
    roles: string[];
    joined_at: string;
  };
}

interface ListInvitesResponse {
  invites: Invite[];
}

export interface CreateInviteInput {
  max_uses?: number;
  expires_in_seconds?: number;
  grant_role_id?: string;
}

export async function createInvite(
  baseUrl: string,
  token: string,
  input: CreateInviteInput,
): Promise<Invite> {
  return apiRequest<Invite>(baseUrl, "/api/v1/invites", {
    method: "POST",
    token,
    body: JSON.stringify(input),
  });
}

export async function listInvites(baseUrl: string, token: string): Promise<Invite[]> {
  const response = await apiRequest<ListInvitesResponse>(baseUrl, "/api/v1/invites", {
    token,
  });
  return response.invites;
}

export async function revokeInvite(baseUrl: string, token: string, code: string): Promise<void> {
  await apiRequest<void>(baseUrl, `/api/v1/invites/${encodeURIComponent(code)}`, {
    method: "DELETE",
    token,
  });
}

export async function getInvitePreview(baseUrl: string, code: string): Promise<InvitePreview> {
  return apiRequest<InvitePreview>(baseUrl, `/api/v1/invites/${encodeURIComponent(code)}`);
}

export async function joinInvite(
  baseUrl: string,
  token: string,
  code: string,
): Promise<JoinInviteResponse> {
  return apiRequest<JoinInviteResponse>(
    baseUrl,
    `/api/v1/invites/${encodeURIComponent(code)}/join`,
    {
      method: "POST",
      token,
    },
  );
}
