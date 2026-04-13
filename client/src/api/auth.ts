export interface ChallengeRequest {
  pubkey: string;
}

export interface ChallengeResponse {
  challenge: string;
}

export interface VerifyRequest {
  pubkey: string;
  signature: string;
}

export interface VerifyResponse {
  token: string;
  user_id: string;
  expires_at: string;
}

export interface AuthFlowInput {
  baseUrl: string;
  pubkey: string;
  sign: (data: string) => Promise<string>;
}

export async function requestChallenge(baseUrl: string, pubkey: string): Promise<ChallengeResponse> {
  const response = await fetch(`${baseUrl}/api/v1/auth/challenge`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ pubkey } satisfies ChallengeRequest),
  });

  if (!response.ok) {
    throw new Error(`challenge failed (${response.status})`);
  }

  return (await response.json()) as ChallengeResponse;
}

export async function verifyChallenge(
  baseUrl: string,
  pubkey: string,
  signature: string,
): Promise<VerifyResponse> {
  const response = await fetch(`${baseUrl}/api/v1/auth/verify`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ pubkey, signature } satisfies VerifyRequest),
  });

  if (!response.ok) {
    throw new Error(`verify failed (${response.status})`);
  }

  return (await response.json()) as VerifyResponse;
}

export async function authenticateWithServer(input: AuthFlowInput): Promise<VerifyResponse> {
  const challenge = await requestChallenge(input.baseUrl, input.pubkey);
  const signature = await input.sign(challenge.challenge);
  return verifyChallenge(input.baseUrl, input.pubkey, signature);
}

export function authHeader(token: string): Record<string, string> {
  return { Authorization: `Bearer ${token}` };
}
