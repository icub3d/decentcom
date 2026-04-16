import React from 'react';
import { useCallback } from 'react';
import type { ReactionCount } from '../stores/serverStore';
import * as reactionsApi from '../api/reactions';

interface ReactionBarProps {
  baseUrl: string;
  channelId: string;
  messageId: string;
  reactions?: ReactionCount[];
  sessionToken: string | null;
  onReactionChange?: () => void;
  showAddButton?: boolean;
  onAddClick?: () => void;
}

export const ReactionBar: React.FC<ReactionBarProps> = ({
  baseUrl,
  channelId,
  messageId,
  reactions = [],
  sessionToken,
  onReactionChange,
  showAddButton = false,
  onAddClick,
}) => {
  const handleToggleReaction = useCallback(
    async (emoji: string, hasReacted: boolean) => {
      if (!sessionToken) return;

      try {
        if (hasReacted) {
          await reactionsApi.removeReaction(baseUrl, sessionToken, channelId, messageId, emoji);
        } else {
          await reactionsApi.addReaction(baseUrl, sessionToken, channelId, messageId, emoji);
        }
        onReactionChange?.();
      } catch (error) {
        console.error('Failed to toggle reaction:', error);
      }
    },
    [baseUrl, channelId, messageId, sessionToken, onReactionChange],
  );

  if (reactions.length === 0 && !showAddButton) {
    return null;
  }

  return (
    <div className="flex flex-wrap gap-1 items-center mt-1">
      {reactions.map((reaction) => (
        <button
          key={reaction.emoji}
          onClick={() => handleToggleReaction(reaction.emoji, reaction.me)}
          className={`inline-flex items-center gap-1 px-2 py-0.5 rounded text-sm transition-colors ${
            reaction.me
              ? 'bg-ctp-mauve text-ctp-crust hover:bg-ctp-pink'
              : 'bg-ctp-surface0 text-ctp-text hover:bg-ctp-surface1'
          }`}
          title={`${reaction.count} user${reaction.count === 1 ? '' : 's'} reacted`}
        >
          <span>{reaction.emoji}</span>
          <span className="text-xs">{reaction.count}</span>
        </button>
      ))}
      {showAddButton && (
        <button
          onClick={onAddClick}
          className="inline-flex items-center justify-center w-6 h-6 rounded bg-ctp-surface0 text-ctp-subtext0 hover:bg-ctp-surface1 transition-colors"
          title="Add reaction"
        >
          <span className="text-sm">+</span>
        </button>
      )}
    </div>
  );
};
