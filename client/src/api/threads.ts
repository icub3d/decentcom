import { apiRequest } from "../services/api";
import type { Message, MessagePage } from "./channels";

export interface CreateThreadResponse {
  thread_id: string;
  parent_message_id: string;
  channel_id: string;
  created_at: string;
  reply_count: number;
  follower_count: number;
}

export interface Thread {
  id: string;
  channel_id: string;
  parent_message_id: string;
  creator_id: string;
  created_at: string;
  last_reply_at: string | null;
  reply_count: number;
  follower_count: number;
  is_following: boolean;
}

export async function createThread(
  address: string,
  token: string,
  channelId: string,
  messageId: string,
): Promise<CreateThreadResponse> {
  return apiRequest<CreateThreadResponse>(
    address,
    `/api/v1/channels/${channelId}/messages/${messageId}/threads`,
    { method: "POST", token },
  );
}

export async function getThread(
  address: string,
  token: string,
  threadId: string,
): Promise<Thread> {
  return apiRequest<Thread>(address, `/api/v1/threads/${threadId}`, { token });
}

export async function listThreadMessages(
  address: string,
  token: string,
  threadId: string,
  before?: string,
  after?: string,
  limit?: number,
): Promise<MessagePage> {
  const params = new URLSearchParams();
  if (before) params.set("before", before);
  if (after) params.set("after", after);
  if (limit) params.set("limit", limit.toString());

  const query = params.toString();
  const path = `/api/v1/threads/${threadId}/messages${query ? `?${query}` : ""}`;
  return apiRequest<MessagePage>(address, path, { token });
}

export async function createThreadMessage(
  address: string,
  token: string,
  threadId: string,
  content: string,
  attachmentIds: string[] = [],
): Promise<Message> {
  return apiRequest<Message>(address, `/api/v1/threads/${threadId}/messages`, {
    method: "POST",
    token,
    body: JSON.stringify({ content, attachment_ids: attachmentIds }),
  });
}

export async function followThread(
  address: string,
  token: string,
  threadId: string,
): Promise<void> {
  return apiRequest<void>(address, `/api/v1/threads/${threadId}/follow`, {
    method: "PUT",
    token,
  });
}

export async function unfollowThread(
  address: string,
  token: string,
  threadId: string,
): Promise<void> {
  return apiRequest<void>(address, `/api/v1/threads/${threadId}/follow`, {
    method: "DELETE",
    token,
  });
}

export async function markThreadRead(
  address: string,
  token: string,
  threadId: string,
): Promise<void> {
  return apiRequest<void>(address, `/api/v1/threads/${threadId}/read`, {
    method: "PUT",
    token,
  });
}
