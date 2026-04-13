import { useState } from "react";
import { IdentityInfo, PublicKeyInfo } from "../hooks/useIdentity";
import { SeedPhrase } from "./SeedPhrase";

interface SetupProps {
  onGenerate: () => Promise<IdentityInfo>;
  onImport: (seedPhrase: string[]) => Promise<PublicKeyInfo>;
  onComplete: () => void;
}

export function Setup({ onGenerate, onImport, onComplete }: SetupProps) {
  const [view, setView] = useState<"choice" | "generate" | "import">("choice");
  const [seedPhrase, setSeedPhrase] = useState<string[]>([]);
  const [importText, setImportText] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleGenerate = async () => {
    try {
      setLoading(true);
      setError(null);
      const info = await onGenerate();
      setSeedPhrase(info.seed_phrase);
      setView("generate");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleImport = async () => {
    const words = importText.trim().split(/\s+/);
    if (words.length !== 24) {
      setError(`Mnemonic must be 24 words. Currently ${words.length} words.`);
      return;
    }
    try {
      setLoading(true);
      setError(null);
      await onImport(words);
      onComplete();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  if (view === "generate") {
    return <SeedPhrase seedPhrase={seedPhrase} onConfirm={onComplete} />;
  }

  if (view === "import") {
    return (
      <div className="max-w-xl mx-auto p-8 space-y-6 bg-ctp-mantle rounded-xl shadow-xl border border-ctp-overlay0 text-ctp-text">
        <h2 className="text-2xl font-bold text-ctp-text">Import Identity</h2>
        <p className="text-ctp-subtext1">Enter your 24-word recovery phrase to restore your identity.</p>
        <textarea
          value={importText}
          onChange={(e) => setImportText(e.target.value)}
          placeholder="Enter 24 words separated by spaces..."
          className="w-full h-40 p-4 bg-ctp-base rounded-lg border border-ctp-overlay0 text-ctp-text font-mono resize-none focus:outline-none focus:ring-2 focus:ring-ctp-blue"
        />
        {error && <p className="text-ctp-red text-sm">{error}</p>}
        <div className="flex gap-4">
          <button
            onClick={() => setView("choice")}
            className="flex-1 py-3 bg-ctp-surface0 hover:bg-ctp-surface1 text-ctp-text rounded-lg font-semibold transition-colors cursor-pointer"
          >
            Back
          </button>
          <button
            onClick={handleImport}
            disabled={loading}
            className="flex-1 py-3 bg-ctp-blue hover:bg-ctp-sapphire disabled:bg-ctp-overlay0 text-ctp-crust rounded-lg font-semibold transition-colors shadow-lg cursor-pointer"
          >
            {loading ? "Importing..." : "Restore Identity"}
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-md mx-auto p-8 space-y-8 bg-ctp-mantle rounded-xl shadow-xl border border-ctp-overlay0 text-ctp-text">
      <div className="text-center space-y-2">
        <h1 className="text-3xl font-bold text-ctp-text">Welcome</h1>
        <p className="text-ctp-subtext0">Set up your decentcom identity to get started.</p>
      </div>
      <div className="space-y-4">
        <button
          onClick={handleGenerate}
          disabled={loading}
          className="w-full py-4 bg-ctp-blue hover:bg-ctp-sapphire disabled:bg-ctp-overlay0 text-ctp-crust rounded-xl font-bold text-lg transition-all shadow-lg hover:scale-[1.02] active:scale-95 cursor-pointer"
        >
          {loading ? "Generating..." : "Create New Identity"}
        </button>
        <button
          onClick={() => setView("import")}
          className="w-full py-4 bg-ctp-surface0 hover:bg-ctp-surface1 text-ctp-text rounded-xl font-bold text-lg transition-all border border-ctp-overlay0 hover:scale-[1.02] active:scale-95 cursor-pointer"
        >
          Import Existing
        </button>
      </div>
      {error && <p className="text-ctp-red text-center text-sm">{error}</p>}
    </div>
  );
}
