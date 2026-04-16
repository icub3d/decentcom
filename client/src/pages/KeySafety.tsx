import { useState } from "react";
import { KeyExport } from "../components/backup/KeyExport";
import type { IdentityInfo } from "../hooks/useIdentity";

interface KeySafetyProps {
  identity: IdentityInfo;
  onComplete: () => void;
}

export function KeySafety({ identity, onComplete }: KeySafetyProps) {
  const [copied, setCopied] = useState(false);
  const [showPhrase, setShowPhrase] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(identity.seed_phrase.join(" "));
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="max-w-xl mx-auto p-8 space-y-6 bg-ctp-mantle rounded-xl shadow-xl border border-ctp-overlay0 text-ctp-text">
      <h2 className="text-2xl font-bold text-ctp-text">Secure Your Identity</h2>

      <div className="bg-ctp-red/20 border border-ctp-red/50 p-4 rounded-lg text-sm">
        <p className="font-semibold text-ctp-red">⚠️ Important</p>
        <p className="text-ctp-text mt-1">
          If you lose access to this device and don&apos;t have a backup, your
          identity is <strong>permanently lost</strong>. You will not be able to
          authenticate to any server. Save at least one of the recovery options
          below.
        </p>
      </div>

      {/* Option 1: Export backup file */}
      <div className="p-4 bg-ctp-base rounded-lg border border-ctp-overlay0 space-y-3">
        <h3 className="text-sm font-semibold text-ctp-text">
          Option 1: Export Backup File
        </h3>
        <p className="text-xs text-ctp-subtext0">
          Save an encrypted backup file that you can use to restore your
          identity on any device.
        </p>
        <KeyExport />
      </div>

      {/* Option 2: Seed phrase */}
      <div className="p-4 bg-ctp-base rounded-lg border border-ctp-overlay0 space-y-3">
        <h3 className="text-sm font-semibold text-ctp-text">
          Option 2: Recovery Phrase
        </h3>
        <p className="text-xs text-ctp-subtext0">
          Your 24-word recovery phrase can restore your identity without a
          backup file. Write it down and store it securely.
        </p>

        {!showPhrase ? (
          <button
            onClick={() => setShowPhrase(true)}
            className="w-full py-2 text-sm rounded border border-dashed border-ctp-overlay0 text-ctp-subtext1 hover:text-ctp-blue hover:border-ctp-blue transition cursor-pointer"
          >
            Reveal Recovery Phrase
          </button>
        ) : (
          <>
            <div
              data-testid="seed-phrase-grid"
              className="grid grid-cols-3 gap-2 p-3 bg-ctp-mantle rounded-lg border border-ctp-overlay0 font-mono text-sm"
            >
              {identity.seed_phrase.map((word, i) => (
                <div key={i} className="flex gap-2">
                  <span className="text-ctp-overlay1 text-right w-6">
                    {i + 1}.
                  </span>
                  <span className="text-ctp-text">{word}</span>
                </div>
              ))}
            </div>
            <button
              onClick={() => void handleCopy()}
              className="w-full py-1.5 text-sm rounded bg-ctp-surface0 hover:bg-ctp-surface1 text-ctp-text transition cursor-pointer"
            >
              {copied ? "✓ Copied!" : "Copy to Clipboard"}
            </button>
          </>
        )}
      </div>

      <button
        onClick={onComplete}
        className="w-full py-3 bg-ctp-blue hover:bg-ctp-sapphire text-ctp-crust rounded-lg font-semibold transition-colors shadow-lg cursor-pointer"
      >
        I&apos;ve saved my key — Continue
      </button>
    </div>
  );
}
