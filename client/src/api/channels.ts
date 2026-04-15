import { apiRequest } from "../services/api";

export interface Channel {
  id: string;
  name: string;
  category_id: string | null;
  position: number;
  type: string;
}

export interface Category {
  id: string;
  name: string;
  position: number;
}

export interface CreateChannelRequest {
  name: string;
  category_id?: string | null;
  position?: number;
}

export interface UpdateChannelRequest {
  name?: string;
  topic?: string | null;
  category_id?: string | null;
  position?: number;
}

export interface CreateCategoryRequest {
  name: string;
  position?: number;
}

export interface UpdateCategoryRequest {
  name?: string;
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

export async function createCategory(
  baseUrl: string,
  token: string,
  req: CreateCategoryRequest
): Promise<Category> {
  return apiRequest<Category>(baseUrl, "/api/v1/categories", {
    method: "POST",
    token,
    body: JSON.stringify(req),
  });
}

export async function updateCategory(
  baseUrl: string,
  token: string,
  categoryId: string,
  req: UpdateCategoryRequest
): Promise<Category> {
  return apiRequest<Category>(baseUrl, `/api/v1/categories/${categoryId}`, {
    method: "PATCH",
    token,
    body: JSON.stringify(req),
  });
}

export async function deleteCategory(
  baseUrl: string,
  token: string,
  categoryId: string
): Promise<void> {
  return apiRequest<void>(baseUrl, `/api/v1/categories/${categoryId}`, {
    method: "DELETE",
    token,
  });
}
