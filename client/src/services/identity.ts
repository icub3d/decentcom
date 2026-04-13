import { invoke } from "@tauri-apps/api/core";

interface PublicKeyInfo {
  pubkey: string;
}

interface SignatureInfo {
  signature: string;
}

export async function getPublicKey(): Promise<string> {
  const info = await invoke<PublicKeyInfo>("get_public_key");
  return info.pubkey;
}

export async function signChallenge(challenge: string): Promise<string> {
  const info = await invoke<SignatureInfo>("sign", { data: challenge });
  return info.signature;
}
