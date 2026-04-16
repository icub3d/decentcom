import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useIdentityStore } from "../stores/identityStore";

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
  const identityRefresh = useIdentityStore((s) => s.refresh);

  const checkIdentity = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const exists = await invoke<boolean>("has_identity");
      setHasIdentity(exists);
      if (exists) {
        const info = await invoke<PublicKeyInfo>("get_public_key");
        setPublicKey(info.pubkey);
        // Sync the identity store with backend state.
        await identityRefresh();
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [identityRefresh]);

  useEffect(() => {
    checkIdentity();
  }, [checkIdentity]);

  const generateIdentity = async (): Promise<IdentityInfo> => {
    try {
      setLoading(true);
      setError(null);
      const info = await invoke<IdentityInfo>("generate_identity");
      // Don't set hasIdentity here — let the caller's onComplete flow
      // trigger refresh() so the safety screen stays mounted.
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
      await identityRefresh();
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
