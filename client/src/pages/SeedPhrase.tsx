interface SeedPhraseProps {
  seedPhrase: string[];
  onConfirm: () => void;
}

export function SeedPhrase({ seedPhrase, onConfirm }: SeedPhraseProps) {
  return (
    <div className="max-w-xl mx-auto p-8 space-y-6 bg-slate-800 rounded-xl shadow-xl border border-slate-700">
      <h2 className="text-2xl font-bold text-slate-100">Your Recovery Phrase</h2>
      <p className="text-slate-300">
        Write down these 24 words in the correct order and keep them safe.
        Anyone who has these words can access your account.
      </p>
      <div
        data-testid="seed-phrase-grid"
        className="grid grid-cols-3 gap-3 p-4 bg-slate-900 rounded-lg border border-slate-700 select-all font-mono"
      >
        {seedPhrase.map((word, i) => (
          <div key={i} className="flex gap-2">
            <span className="text-slate-500 text-right w-6">{i + 1}.</span>
            <span className="text-slate-100">{word}</span>
          </div>
        ))}
      </div>
      <div className="bg-amber-900/30 border border-amber-800/50 p-4 rounded-lg text-amber-200 text-sm">
        ⚠️ <strong>WARNING:</strong> There is no "forgot password" feature. If
        you lose this phrase, your account cannot be recovered.
      </div>
      <button
        onClick={onConfirm}
        className="w-full py-3 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-semibold transition-colors shadow-lg cursor-pointer"
      >
        I've written it down
      </button>
    </div>
  );
}
