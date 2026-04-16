import { useState, useCallback, useEffect } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import {
  keyExport,
  keyExportValidatePassphrase,
  type PassphraseValidation,
} from "../../services/identity";

const strengthColors: Record<string, string> = {
  tooshort: "bg-ctp-red",
  weak: "bg-ctp-peach",
  fair: "bg-ctp-yellow",
  strong: "bg-ctp-green",
};

const strengthLabels: Record<string, string> = {
  tooshort: "Too short",
  weak: "Weak",
  fair: "Fair",
  strong: "Strong",
};

export function KeyExport() {
  const [passphrase, setPassphrase] = useState("");
  const [confirm, setConfirm] = useState("");
  const [validation, setValidation] = useState<PassphraseValidation | null>(
    null,
  );
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const [exporting, setExporting] = useState(false);

  const validate = useCallback(async (pass: string) => {
    if (!pass) {
      setValidation(null);
      return;
    }
    try {
      const result = await keyExportValidatePassphrase(pass);
      setValidation(result);
    } catch {
      setValidation(null);
    }
  }, []);

  useEffect(() => {
    const timer = setTimeout(() => void validate(passphrase), 200);
    return () => clearTimeout(timer);
  }, [passphrase, validate]);

  const canExport =
    passphrase.length > 0 &&
    passphrase === confirm &&
    validation !== null &&
    validation.strength !== "tooshort" &&
    !exporting;

  const handleExport = async () => {
    setError(null);
    setSuccess(false);
    try {
      const path = await save({
        title: "Save Key Backup",
        defaultPath: "identity-backup.dckb",
        filters: [{ name: "DecentCom Key Backup", extensions: ["dckb"] }],
      });
      if (!path) return;

      setExporting(true);
      await keyExport(passphrase, path);
      setSuccess(true);
      setPassphrase("");
      setConfirm("");
    } catch (e) {
      setError(String(e));
    } finally {
      setExporting(false);
    }
  };

  return (
    <div className="space-y-3">
      <h4 className="text-sm font-semibold text-ctp-subtext1">
        Export Key Backup
      </h4>
      <p className="text-xs text-ctp-subtext0">
        Encrypt your private key with a passphrase. If you lose access to this
        device, you can restore your identity using this backup file.
      </p>

      <div className="space-y-2">
        <input
          type="password"
          placeholder="Passphrase"
          value={passphrase}
          onChange={(e) => {
            setPassphrase(e.target.value);
            setSuccess(false);
          }}
          className="w-full bg-ctp-surface0 text-ctp-text rounded px-3 py-1.5 text-sm outline-none focus:ring-1 focus:ring-ctp-blue"
        />

        {validation && (
          <div className="space-y-1">
            <div className="flex items-center gap-2">
              <div className="flex-1 h-1.5 rounded bg-ctp-surface1 overflow-hidden">
                <div
                  className={`h-full rounded transition-all ${strengthColors[validation.strength] ?? "bg-ctp-surface1"}`}
                  style={{
                    width:
                      validation.strength === "tooshort"
                        ? "10%"
                        : validation.strength === "weak"
                          ? "33%"
                          : validation.strength === "fair"
                            ? "66%"
                            : "100%",
                  }}
                />
              </div>
              <span className="text-[10px] text-ctp-subtext0 w-16">
                {strengthLabels[validation.strength] ?? ""}
              </span>
            </div>
            <p className="text-[10px] text-ctp-subtext0">
              {validation.feedback}
            </p>
          </div>
        )}

        <input
          type="password"
          placeholder="Confirm passphrase"
          value={confirm}
          onChange={(e) => {
            setConfirm(e.target.value);
            setSuccess(false);
          }}
          className="w-full bg-ctp-surface0 text-ctp-text rounded px-3 py-1.5 text-sm outline-none focus:ring-1 focus:ring-ctp-blue"
        />

        {confirm && passphrase !== confirm && (
          <p className="text-[10px] text-ctp-red">
            Passphrases do not match.
          </p>
        )}
      </div>

      {error && <p className="text-xs text-ctp-red">{error}</p>}
      {success && (
        <p className="text-xs text-ctp-green">
          ✓ Backup saved. Store it somewhere safe!
        </p>
      )}

      <button
        disabled={!canExport}
        onClick={() => void handleExport()}
        className="w-full py-1.5 text-sm rounded bg-ctp-blue text-ctp-crust font-medium disabled:opacity-40 hover:bg-ctp-blue/80 transition"
      >
        {exporting ? "Encrypting…" : "Export Backup"}
      </button>
    </div>
  );
}
