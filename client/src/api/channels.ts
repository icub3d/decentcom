import { apiRequest } from "../services/api";
import type { Attachment } from "./media";
import type { ReactionSummary } from "./reactions";

export interface Channel {
  id: string;
  name: string;
  category: string | null;
  position: number;
  type: string;
}

export interface ThreadSummary {
  thread_id: string;
  reply_count: number;
  last_reply_at: string | null;
  last_reply_author_id: string | null;
}

export interface Message {
  id: string;
  channel_id: string;
  author_id: string;
  content: string;
  created_at: string;
  edited_at: string | null;
  deleted: boolean;
  attachments: Attachment[];
  reactions: ReactionSummary[];
  thread: ThreadSummary | null;
}

export interface MessagePage {
  messages: Message[];
  has_more: boolean;
}

export interface CreateChannelRequest {
  name: string;
  category?: string | null;
  position?: number;
}

export interface UpdateChannelRequest {
  name?: string;
  topic?: string | null;
  category?: string | null;
  position?: number;
}

export async function createChannel(
  baseUrl: string,
  token: string,
  req: CreateChannelRequest
): Promise<Channel> {
  return apiRequest<Channel>(baseUrl, "/api/v1/channels", {
    method: "POST",
    token,
    body: JSON.stringify(req),
  });
}

export async function updateChannel(
  baseUrl: string,
  token: string,
  channelId: string,
  req: UpdateChannelRequest
): Promise<Channel> {
  return apiRequest<Channel>(baseUrl, `/api/v1/channels/${channelId}`, {
    method: "PATCH",
    token,
    body: JSON.stringify(req),
  });
}

export async function deleteChannel(
  baseUrl: string,
  token: string,
  channelId: string
): Promise<void> {
  return apiRequest<void>(baseUrl, `/api/v1/channels/${channelId}`, {
    method: "DELETE",
    token,
  });
}
