import { apiRequest } from "../services/api";

export interface ReactionSummary {
  emoji: string;
  count: number;
  me: boolean;
}

export interface ReactorEntry {
  id: string;
  display_name: string | null;
}

export interface ListReactorsResponse {
  users: ReactorEntry[];
  total: number;
}

export async function putReaction(
  address: string,
  token: string,
  channelId: string,
  messageId: string,
  emoji: string,
): Promise<void> {
  const encoded = encodeURIComponent(emoji);
  await apiRequest<void>(
    address,
    `/api/v1/channels/${channelId}/messages/${messageId}/reactions/${encoded}`,
    { token, method: "PUT" },
  );
}

export async function deleteOwnReaction(
  address: string,
  token: string,
  channelId: string,
  messageId: string,
): Promise<void> {
  await apiRequest<void>(
    address,
    `/api/v1/channels/${channelId}/messages/${messageId}/reactions`,
    { token, method: "DELETE" },
  );
}

export async function deleteUserReaction(
  address: string,
  token: string,
  channelId: string,
  messageId: string,
  userId: string,
): Promise<void> {
  await apiRequest<void>(
    address,
    `/api/v1/channels/${channelId}/messages/${messageId}/reactions/users/${userId}`,
    { token, method: "DELETE" },
  );
}

export async function listReactors(
  address: string,
  token: string,
  channelId: string,
  messageId: string,
  emoji: string,
): Promise<ListReactorsResponse> {
  const encoded = encodeURIComponent(emoji);
  return apiRequest<ListReactorsResponse>(
    address,
    `/api/v1/channels/${channelId}/messages/${messageId}/reactions/${encoded}`,
    { token },
  );
}
