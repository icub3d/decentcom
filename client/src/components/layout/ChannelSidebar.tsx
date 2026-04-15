import { useState, useMemo } from "react";

import { MANAGE_CHANNELS, usePermissions } from "../../hooks/usePermissions";
import type { Channel } from "../../stores/serverStore";
import { useServerStore } from "../../stores/serverStore";
import { CreateChannelDialog } from "../channels/CreateChannelDialog";
import { EditChannelDialog } from "../channels/EditChannelDialog";
import { StatusIndicator } from "../connection/StatusIndicator";

interface ChannelSidebarProps {
  channels: Channel[];
  currentChannelId: string | null;
  status: "connecting" | "connected" | "disconnected";
  onSelectChannel: (id: string) => void;
}

type DialogState =
  | { kind: "none" }
  | { kind: "createChannel" }
  | { kind: "editChannel"; channel: Channel };

export function ChannelSidebar({
  channels,
  currentChannelId,
  status,
  onSelectChannel,
}: ChannelSidebarProps) {
  const permissions = usePermissions();
  const canManage = permissions.has(MANAGE_CHANNELS);
  const [dialog, setDialog] = useState<DialogState>({ kind: "none" });

  const {
    createChannel,
    updateChannel,
    deleteChannel,
  } = useServerStore();

  // Group channels by category string, sorted alphabetically by category name.
  // Uncategorized channels (null category) go under "General".
  const grouped = useMemo(() => {
    const groups: Record<string, Channel[]> = {};
    for (const ch of channels) {
      const cat = ch.category ?? "General";
      if (!groups[cat]) groups[cat] = [];
      groups[cat].push(ch);
    }
    // Sort category names alphabetically, but keep "General" last
    const sortedKeys = Object.keys(groups).sort((a, b) => {
      if (a === "General") return 1;
      if (b === "General") return -1;
      return a.localeCompare(b);
    });
    return sortedKeys.map((name) => ({ name, channels: groups[name] }));
  }, [channels]);

  // Collect existing category names for the create dialog
  const existingCategories = useMemo(() => {
    const names = new Set<string>();
    for (const ch of channels) {
      if (ch.category) names.add(ch.category);
    }
    return Array.from(names).sort();
  }, [channels]);

  const channelButtonClass = (id: string) =>
    `flex-1 min-w-0 rounded-md px-3 py-2 text-left text-sm transition ${
      currentChannelId === id
        ? "bg-ctp-blue/20 text-ctp-blue"
        : "text-ctp-subtext1 hover:bg-ctp-surface0"
    }`;

  return (
    <aside className="w-72 border-r border-ctp-overlay0 bg-ctp-mantle/80 p-4 overflow-auto">
      <div className="mb-4 flex items-center justify-between">
        <h2 className="text-sm font-bold uppercase tracking-wide text-ctp-subtext0">Channels</h2>
        <div className="flex items-center gap-2">
          {canManage && (
            <button
              onClick={() => setDialog({ kind: "createChannel" })}
              title="Create Channel"
              className="rounded-md px-1.5 py-0.5 text-sm text-ctp-subtext0 hover:bg-ctp-surface0 hover:text-ctp-blue transition"
            >
              +
            </button>
          )}
          <StatusIndicator status={status} />
        </div>
      </div>

      <div className="space-y-4">
        {grouped.map((group) => (
          <section key={group.name} className="space-y-1">
            <h3 className="text-xs font-semibold uppercase tracking-wide text-ctp-overlay1">
              {group.name}
            </h3>
            {group.channels.map((channel) => (
              <div key={channel.id} className="flex items-center group/ch">
                <button
                  onClick={() => onSelectChannel(channel.id)}
                  className={channelButtonClass(channel.id)}
                >
                  <span className="truncate block">#{channel.name}</span>
                </button>
                {canManage && (
                  <button
                    onClick={() => setDialog({ kind: "editChannel", channel })}
                    title={`Edit ${channel.name}`}
                    className="shrink-0 rounded-md px-1 py-0.5 text-xs text-ctp-overlay0 opacity-0 group-hover/ch:opacity-100 hover:text-ctp-blue hover:bg-ctp-surface0 transition"
                  >
                    ⚙
                  </button>
                )}
              </div>
            ))}
          </section>
        ))}
      </div>

      {dialog.kind === "createChannel" && (
        <CreateChannelDialog
          existingCategories={existingCategories}
          onClose={() => setDialog({ kind: "none" })}
          onCreate={async (name, category, position) => {
            await createChannel({ name, category, position });
          }}
        />
      )}

      {dialog.kind === "editChannel" && (
        <EditChannelDialog
          channel={dialog.channel}
          existingCategories={existingCategories}
          onClose={() => setDialog({ kind: "none" })}
          onUpdate={async (channelId, name, category, position, topic) => {
            await updateChannel(channelId, { name, category, position, topic });
          }}
          onDelete={async (channelId) => {
            await deleteChannel(channelId);
          }}
        />
      )}
    </aside>
  );
}
