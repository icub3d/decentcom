import { apiRequest } from "../services/api";

export interface MemberRole {
  id: string;
  name: string;
  color: string | null;
  position: number;
}

export interface Member {
  user_id: string;
  pubkey: string;
  joined_at: string;
  roles: MemberRole[];
}

export interface Ban {
  pubkey: string;
  banned_by: string;
  reason: string | null;
  banned_at: string;
}

export interface AllowlistEntry {
  pubkey: string;
  added_by: string;
  added_at: string;
}

interface JoinMemberResponse {
  member: Member;
}

interface ListMembersResponse {
  members: Member[];
}

interface ListBansResponse {
  bans: Ban[];
}

interface ListAllowlistResponse {
  entries: AllowlistEntry[];
}

export async function joinServer(baseUrl: string, token: string): Promise<Member> {
  const response = await apiRequest<JoinMemberResponse>(baseUrl, "/api/v1/members/join", {
    method: "POST",
    token,
  });
  return response.member;
}

export async function leaveServer(baseUrl: string, token: string): Promise<void> {
  await apiRequest<void>(baseUrl, "/api/v1/members/me", { method: "DELETE", token });
}

export async function listMembers(baseUrl: string, token: string): Promise<Member[]> {
  const response = await apiRequest<ListMembersResponse>(baseUrl, "/api/v1/members", { token });
  return response.members;
}

export async function kickMember(baseUrl: string, token: string, pubkey: string): Promise<void> {
  await apiRequest<void>(
    baseUrl,
    `/api/v1/members/${encodeURIComponent(pubkey)}/kick`,
    { method: "POST", token },
  );
}

export async function banMember(
  baseUrl: string,
  token: string,
  pubkey: string,
  reason?: string,
): Promise<Ban> {
  return apiRequest<Ban>(
    baseUrl,
    `/api/v1/members/${encodeURIComponent(pubkey)}/ban`,
    {
      method: "POST",
      token,
      body: JSON.stringify({ reason }),
    },
  );
}

export async function listBans(baseUrl: string, token: string): Promise<Ban[]> {
  const response = await apiRequest<ListBansResponse>(baseUrl, "/api/v1/bans", { token });
  return response.bans;
}

export async function unbanMember(baseUrl: string, token: string, pubkey: string): Promise<void> {
  await apiRequest<void>(baseUrl, `/api/v1/bans/${encodeURIComponent(pubkey)}`, {
    method: "DELETE",
    token,
  });
}

export async function listAllowlist(baseUrl: string, token: string): Promise<AllowlistEntry[]> {
  const response = await apiRequest<ListAllowlistResponse>(baseUrl, "/api/v1/allowlist", { token });
  return response.entries;
}

export async function addAllowlist(baseUrl: string, token: string, pubkey: string): Promise<AllowlistEntry> {
  return apiRequest<AllowlistEntry>(baseUrl, "/api/v1/allowlist", {
    method: "POST",
    token,
    body: JSON.stringify({ pubkey }),
  });
}

export async function removeAllowlist(baseUrl: string, token: string, pubkey: string): Promise<void> {
  await apiRequest<void>(baseUrl, `/api/v1/allowlist/${encodeURIComponent(pubkey)}`, {
    method: "DELETE",
    token,
  });
}
