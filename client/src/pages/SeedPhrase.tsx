interface SeedPhraseProps {
  seedPhrase: string[];
  onConfirm: () => void;
}

export function SeedPhrase({ seedPhrase, onConfirm }: SeedPhraseProps) {
  return (
    <div className="max-w-xl mx-auto p-8 space-y-6 bg-ctp-mantle rounded-xl shadow-xl border border-ctp-overlay0 text-ctp-text">
      <h2 className="text-2xl font-bold text-ctp-text">Your Recovery Phrase</h2>
      <p className="text-ctp-subtext1">
        Write down these 24 words in the correct order and keep them safe.
        Anyone who has these words can access your account.
      </p>
      <div
        data-testid="seed-phrase-grid"
        className="grid grid-cols-3 gap-3 p-4 bg-ctp-base rounded-lg border border-ctp-overlay0 select-all font-mono"
      >
        {seedPhrase.map((word, i) => (
          <div key={i} className="flex gap-2">
            <span className="text-ctp-overlay1 text-right w-6">{i + 1}.</span>
            <span className="text-ctp-text">{word}</span>
          </div>
        ))}
      </div>
      <div className="bg-ctp-yellow/20 border border-ctp-yellow/50 p-4 rounded-lg text-ctp-yellow text-sm">
        ⚠️ <strong>WARNING:</strong> There is no "forgot password" feature. If
        you lose this phrase, your account cannot be recovered.
      </div>
      <button
        onClick={onConfirm}
        className="w-full py-3 bg-ctp-blue hover:bg-ctp-sapphire text-ctp-crust rounded-lg font-semibold transition-colors shadow-lg cursor-pointer"
      >
        I've written it down
      </button>
    </div>
  );
}
