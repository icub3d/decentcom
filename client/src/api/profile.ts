import { apiRequest } from "../services/api";
import type { Member } from "./members";

export interface ProfileUpdateRequest {
  display_name?: string;
}

export interface MemberProfile {
  user_id: string;
  pubkey: string;
  display_name: string | null;
  avatar_hash: string | null;
  joined_at: string;
  roles: { id: string; name: string; color: string | null; position: number }[];
}

export async function updateProfile(
  baseUrl: string,
  token: string,
  data: ProfileUpdateRequest,
): Promise<Member> {
  return apiRequest<Member>(baseUrl, "/api/v1/profile", {
    method: "PATCH",
    token,
    body: JSON.stringify(data),
  });
}

export async function uploadAvatar(
  baseUrl: string,
  token: string,
  file: File,
): Promise<Member> {
  const formData = new FormData();
  formData.append("avatar", file);

  const response = await fetch(`${baseUrl}/api/v1/profile/avatar`, {
    method: "PUT",
    headers: {
      Authorization: `Bearer ${token}`,
    },
    body: formData,
  });

  if (!response.ok) {
    let message = `request failed (${response.status})`;
    try {
      const body = (await response.json()) as { error?: string };
      if (body?.error) {
        message = body.error;
      }
    } catch {
      // ignore
    }
    throw new Error(message);
  }

  return (await response.json()) as Member;
}

export async function deleteAvatar(
  baseUrl: string,
  token: string,
): Promise<void> {
  await apiRequest<void>(baseUrl, "/api/v1/profile/avatar", {
    method: "DELETE",
    token,
  });
}

export async function getMemberProfile(
  baseUrl: string,
  token: string,
  pubkey: string,
): Promise<MemberProfile> {
  return apiRequest<MemberProfile>(
    baseUrl,
    `/api/v1/members/${encodeURIComponent(pubkey)}/profile`,
    { token },
  );
}

export function avatarUrl(baseUrl: string, hash: string): string {
  return `${baseUrl}/api/v1/media/${hash}`;
}
