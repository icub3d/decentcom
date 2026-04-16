import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  keyImport,
  keyBackupReadPubkey,
} from "../../services/identity";

function shortPubkey(pubkey: string): string {
  if (pubkey.length <= 12) return pubkey;
  return `${pubkey.slice(0, 6)}…${pubkey.slice(-4)}`;
}

interface KeyImportProps {
  onImported?: (pubkey: string) => void;
}

export function KeyImport({ onImported }: KeyImportProps) {
  const [filePath, setFilePath] = useState<string | null>(null);
  const [backupPubkey, setBackupPubkey] = useState<string | null>(null);
  const [passphrase, setPassphrase] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [importing, setImporting] = useState(false);

  const handlePickFile = async () => {
    setError(null);
    setSuccess(null);
    setBackupPubkey(null);
    try {
      const result = await open({
        title: "Select Key Backup",
        multiple: false,
        filters: [{ name: "DecentCom Key Backup", extensions: ["dckb"] }],
      });
      if (!result) return;
      setFilePath(result);

      const info = await keyBackupReadPubkey(result);
      setBackupPubkey(info.pubkey);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleImport = async () => {
    if (!filePath || !passphrase) return;
    setError(null);
    setSuccess(null);
    setImporting(true);
    try {
      const result = await keyImport(passphrase, filePath);
      setSuccess(result.pubkey);
      setPassphrase("");
      setFilePath(null);
      setBackupPubkey(null);
      onImported?.(result.pubkey);
    } catch (e) {
      setError(String(e));
    } finally {
      setImporting(false);
    }
  };

  return (
    <div className="space-y-3">
      <h4 className="text-sm font-semibold text-ctp-subtext1">
        Import Key Backup
      </h4>
      <p className="text-xs text-ctp-subtext0">
        Restore an identity from a .dckb backup file. You'll need the
        passphrase you used when exporting.
      </p>

      <button
        onClick={() => void handlePickFile()}
        className="w-full py-1.5 text-sm rounded border border-dashed border-ctp-overlay0 text-ctp-subtext1 hover:text-ctp-blue hover:border-ctp-blue transition"
      >
        {filePath ? `Selected: ${filePath.split("/").pop()}` : "Choose .dckb file…"}
      </button>

      {backupPubkey && (
        <p className="text-xs text-ctp-subtext0">
          Identity in file:{" "}
          <span className="font-mono text-ctp-text">
            {shortPubkey(backupPubkey)}
          </span>
        </p>
      )}

      {filePath && (
        <input
          type="password"
          placeholder="Passphrase"
          value={passphrase}
          onChange={(e) => {
            setPassphrase(e.target.value);
            setError(null);
          }}
          className="w-full bg-ctp-surface0 text-ctp-text rounded px-3 py-1.5 text-sm outline-none focus:ring-1 focus:ring-ctp-blue"
        />
      )}

      {error && <p className="text-xs text-ctp-red">{error}</p>}
      {success && (
        <p className="text-xs text-ctp-green">
          ✓ Identity restored: {shortPubkey(success)}
        </p>
      )}

      {filePath && (
        <button
          disabled={!passphrase || importing}
          onClick={() => void handleImport()}
          className="w-full py-1.5 text-sm rounded bg-ctp-green text-ctp-crust font-medium disabled:opacity-40 hover:bg-ctp-green/80 transition"
        >
          {importing ? "Decrypting…" : "Import Identity"}
        </button>
      )}

      <p className="text-[10px] text-ctp-overlay1">
        ⚠ If you lose your private key and don't have a backup, your identity
        cannot be recovered. Keep backup files in a secure location.
      </p>
    </div>
  );
}
