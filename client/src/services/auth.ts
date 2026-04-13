import { authenticateWithServer } from "../api/auth";
import { getPublicKey, signChallenge } from "./identity";

export interface AuthSession {
  token: string;
  userId: string;
  expiresAt: string;
  pubkey: string;
}

export async function authenticateServer(baseUrl: string): Promise<AuthSession> {
  const pubkey = await getPublicKey();
  const result = await authenticateWithServer({
    baseUrl,
    pubkey,
    sign: signChallenge,
  });

  return {
    token: result.token,
    userId: result.user_id,
    expiresAt: result.expires_at,
    pubkey,
  };
}
