import { apiRequest } from '../services/api';

export interface ReactionUser {
  id: string;
  display_name?: string;
}

export interface ReactionListResponse {
  users: ReactionUser[];
  total: number;
}

export async function addReaction(
  baseUrl: string,
  token: string,
  channelId: string,
  messageId: string,
  emoji: string,
): Promise<void> {
  return apiRequest<void>(
    baseUrl,
    `/api/v1/channels/${channelId}/messages/${messageId}/reactions/${encodeURIComponent(emoji)}`,
    {
      method: 'PUT',
      token,
    },
  );
}

export async function removeReaction(
  baseUrl: string,
  token: string,
  channelId: string,
  messageId: string,
  emoji: string,
): Promise<void> {
  return apiRequest<void>(
    baseUrl,
    `/api/v1/channels/${channelId}/messages/${messageId}/reactions/${encodeURIComponent(emoji)}`,
    {
      method: 'DELETE',
      token,
    },
  );
}

export async function removeUserReaction(
  baseUrl: string,
  token: string,
  channelId: string,
  messageId: string,
  emoji: string,
  userId: string,
): Promise<void> {
  return apiRequest<void>(
    baseUrl,
    `/api/v1/channels/${channelId}/messages/${messageId}/reactions/${encodeURIComponent(emoji)}/${userId}`,
    {
      method: 'DELETE',
      token,
    },
  );
}

export async function listReactionUsers(
  baseUrl: string,
  token: string,
  channelId: string,
  messageId: string,
  emoji: string,
): Promise<ReactionListResponse> {
  return apiRequest<ReactionListResponse>(
    baseUrl,
    `/api/v1/channels/${channelId}/messages/${messageId}/reactions/${encodeURIComponent(emoji)}`,
    {
      method: 'GET',
      token,
    },
  );
}
