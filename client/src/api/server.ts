import { apiRequest } from "../services/api";

export interface ServerInfo {
  name: string;
  description: string | null;
  membership_mode: string;
}

export async function getServerInfo(baseUrl: string): Promise<ServerInfo> {
  return apiRequest<ServerInfo>(baseUrl, "/api/v1/server/info");
}
