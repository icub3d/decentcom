import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface IdentityInfo {
  pubkey: string;
  seed_phrase: string[];
}

export interface PublicKeyInfo {
  pubkey: string;
}

export interface SignatureInfo {
  signature: string;
}

export function useIdentity() {
  const [hasIdentity, setHasIdentity] = useState<boolean | null>(null);
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const checkIdentity = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const exists = await invoke<boolean>("has_identity");
      setHasIdentity(exists);
      if (exists) {
        const info = await invoke<PublicKeyInfo>("get_public_key");
        setPublicKey(info.pubkey);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    checkIdentity();
  }, [checkIdentity]);

  const generateIdentity = async (): Promise<IdentityInfo> => {
    try {
      setLoading(true);
      setError(null);
      const info = await invoke<IdentityInfo>("generate_identity");
      setHasIdentity(true);
      setPublicKey(info.pubkey);
      return info;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
      throw new Error(msg);
    } finally {
      setLoading(false);
    }
  };

  const importIdentity = async (seedPhrase: string[]): Promise<PublicKeyInfo> => {
    try {
      setLoading(true);
      setError(null);
      const info = await invoke<PublicKeyInfo>("import_identity", { seedPhrase });
      setHasIdentity(true);
      setPublicKey(info.pubkey);
      return info;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
      throw new Error(msg);
    } finally {
      setLoading(false);
    }
  };

  const sign = async (data: string): Promise<string> => {
    try {
      setError(null);
      const info = await invoke<SignatureInfo>("sign", { data });
      return info.signature;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
      throw new Error(msg);
    }
  };

  return {
    hasIdentity,
    publicKey,
    loading,
    error,
    generateIdentity,
    importIdentity,
    sign,
    refresh: checkIdentity,
  };
}
