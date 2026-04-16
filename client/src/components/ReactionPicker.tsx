import React, { useCallback, useState } from 'react';

interface ReactionPickerProps {
  onSelect: (emoji: string) => void;
  onClose: () => void;
}

const COMMON_EMOJIS = [
  '👍', '👎', '😂', '🤔', '❤️', '🔥', '✨', '👌',
  '😍', '😮', '😢', '😡', '🎉', '🚀', '💯', '👏',
  '🙏', '👌', '💪', '🤝', '🤷', '😅', '😳', '🤗',
];

export const ReactionPicker: React.FC<ReactionPickerProps> = ({ onSelect, onClose }) => {
  const [customEmoji, setCustomEmoji] = useState('');

  const handleEmojiClick = useCallback(
    (emoji: string) => {
      onSelect(emoji);
      onClose();
    },
    [onSelect, onClose],
  );

  const handleCustomEmojiSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      if (customEmoji.trim()) {
        onSelect(customEmoji.trim());
        onClose();
      }
    },
    [customEmoji, onSelect, onClose],
  );

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-ctp-base rounded-lg shadow-lg p-4 max-w-sm w-full mx-4">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-lg font-semibold text-ctp-text">Add Reaction</h2>
          <button
            onClick={onClose}
            className="text-ctp-subtext0 hover:text-ctp-text"
          >
            ✕
          </button>
        </div>

        <div className="mb-4">
          <p className="text-sm text-ctp-subtext0 mb-3">Common reactions:</p>
          <div className="grid grid-cols-8 gap-2">
            {COMMON_EMOJIS.map((emoji) => (
              <button
                key={emoji}
                onClick={() => handleEmojiClick(emoji)}
                className="text-2xl hover:bg-ctp-surface0 p-1 rounded transition-colors"
              >
                {emoji}
              </button>
            ))}
          </div>
        </div>

        <div className="border-t border-ctp-surface0 pt-4">
          <form onSubmit={handleCustomEmojiSubmit}>
            <label className="block text-sm text-ctp-subtext0 mb-2">
              Or enter a custom emoji:
            </label>
            <div className="flex gap-2">
              <input
                type="text"
                value={customEmoji}
                onChange={(e) => setCustomEmoji(e.target.value)}
                placeholder="Enter emoji or text"
                className="flex-1 px-3 py-2 bg-ctp-surface0 border border-ctp-surface1 rounded text-ctp-text placeholder-ctp-subtext1 focus:outline-none focus:border-ctp-mauve"
                maxLength={10}
              />
              <button
                type="submit"
                className="px-4 py-2 bg-ctp-mauve text-ctp-crust rounded hover:bg-ctp-pink transition-colors font-medium"
              >
                Add
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
};
