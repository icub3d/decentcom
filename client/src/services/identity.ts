import { invoke } from "@tauri-apps/api/core";

interface PublicKeyInfo {
  pubkey: string;
}

interface SignatureInfo {
  signature: string;
}

export interface AccountInfo {
  pubkey: string;
  label: string | null;
  active: boolean;
}

export async function getPublicKey(): Promise<string> {
  const info = await invoke<PublicKeyInfo>("get_public_key");
  return info.pubkey;
}

export async function signChallenge(challenge: string): Promise<string> {
  const info = await invoke<SignatureInfo>("sign", { data: challenge });
  return info.signature;
}

export async function listAccounts(): Promise<AccountInfo[]> {
  return invoke<AccountInfo[]>("list_accounts");
}

export async function setActiveAccount(pubkey: string): Promise<void> {
  await invoke("set_active_account", { pubkey });
}

export async function deleteAccount(pubkey: string): Promise<void> {
  await invoke("delete_account", { pubkey });
}

export async function renameAccount(pubkey: string, label: string): Promise<void> {
  await invoke("rename_account", { pubkey, label });
}
