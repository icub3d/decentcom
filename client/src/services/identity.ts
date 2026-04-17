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

// Key backup / recovery

export interface PassphraseValidation {
  strength: "tooshort" | "weak" | "fair" | "strong";
  entropy_bits: number;
  feedback: string;
}

export async function keyExportValidatePassphrase(
  pass: string,
): Promise<PassphraseValidation> {
  return invoke<PassphraseValidation>("key_export_validate_passphrase", {
    pass,
  });
}

export async function keyExport(
  passphrase: string,
  path: string,
  metadata?: string,
): Promise<void> {
  await invoke("key_export", { passphrase, path, metadata: metadata ?? null });
}

export async function keyImport(
  passphrase: string,
  path: string,
): Promise<{ pubkey: string; metadata: string | null }> {
  return invoke<{ pubkey: string; metadata: string | null }>("key_import", {
    passphrase,
    path,
  });
}

export async function keyBackupReadPubkey(
  path: string,
): Promise<{ pubkey: string }> {
  return invoke<{ pubkey: string }>("key_backup_read_pubkey", { path });
}
