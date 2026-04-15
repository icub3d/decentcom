import { useState } from "react";

import { MANAGE_CHANNELS, usePermissions } from "../../hooks/usePermissions";
import type { Category, Channel } from "../../stores/serverStore";
import { useServerStore } from "../../stores/serverStore";
import { CreateCategoryDialog } from "../channels/CreateCategoryDialog";
import { CreateChannelDialog } from "../channels/CreateChannelDialog";
import { EditCategoryDialog } from "../channels/EditCategoryDialog";
import { EditChannelDialog } from "../channels/EditChannelDialog";
import { StatusIndicator } from "../connection/StatusIndicator";

interface ChannelSidebarProps {
  channels: Channel[];
  categories: Category[];
  currentChannelId: string | null;
  status: "connecting" | "connected" | "disconnected";
  onSelectChannel: (id: string) => void;
}

type DialogState =
  | { kind: "none" }
  | { kind: "createChannel" }
  | { kind: "createCategory" }
  | { kind: "editChannel"; channel: Channel }
  | { kind: "editCategory"; category: Category };

export function ChannelSidebar({
  channels,
  categories,
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
    createCategory,
    updateCategory,
    deleteCategory,
  } = useServerStore();

  const uncategorized = channels.filter((ch) => !ch.category_id);

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
            <div className="flex gap-1">
              <button
                onClick={() => setDialog({ kind: "createChannel" })}
                title="Create Channel"
                className="rounded-md px-1.5 py-0.5 text-xs text-ctp-subtext0 hover:bg-ctp-surface0 hover:text-ctp-blue"
              >
                + #
              </button>
              <button
                onClick={() => setDialog({ kind: "createCategory" })}
                title="Create Category"
                className="rounded-md px-1.5 py-0.5 text-xs text-ctp-subtext0 hover:bg-ctp-surface0 hover:text-ctp-blue"
              >
                + ▸
              </button>
            </div>
          )}
          <StatusIndicator status={status} />
        </div>
      </div>

      <div className="space-y-4">
        {categories.map((category) => {
          const grouped = channels.filter((ch) => ch.category_id === category.id);
          return (
            <section key={category.id} className="space-y-1">
              <div className="flex items-center justify-between group">
                <h3 className="text-xs font-semibold uppercase tracking-wide text-ctp-overlay1">
                  {category.name}
                </h3>
                {canManage && (
                  <button
                    onClick={() => setDialog({ kind: "editCategory", category })}
                    title={`Edit ${category.name}`}
                    className="rounded-md px-1 py-0.5 text-xs text-ctp-overlay0 opacity-0 group-hover:opacity-100 hover:text-ctp-blue hover:bg-ctp-surface0 transition"
                  >
                    ⚙
                  </button>
                )}
              </div>
              {grouped.map((channel) => (
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
          );
        })}

        {(!!uncategorized.length || canManage) && (
          <section className="space-y-1">
            <h3 className="text-xs font-semibold uppercase tracking-wide text-ctp-overlay1">General</h3>
            {uncategorized.map((channel) => (
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
        )}
      </div>

      {dialog.kind === "createChannel" && (
        <CreateChannelDialog
          categories={categories}
          onClose={() => setDialog({ kind: "none" })}
          onCreate={async (name, categoryId, position) => {
            await createChannel({ name, category_id: categoryId, position });
          }}
        />
      )}

      {dialog.kind === "createCategory" && (
        <CreateCategoryDialog
          onClose={() => setDialog({ kind: "none" })}
          onCreate={async (name, position) => {
            await createCategory({ name, position });
          }}
        />
      )}

      {dialog.kind === "editChannel" && (
        <EditChannelDialog
          channel={dialog.channel}
          categories={categories}
          onClose={() => setDialog({ kind: "none" })}
          onUpdate={async (channelId, name, categoryId, position, topic) => {
            await updateChannel(channelId, { name, category_id: categoryId, position, topic });
          }}
          onDelete={async (channelId) => {
            await deleteChannel(channelId);
          }}
        />
      )}

      {dialog.kind === "editCategory" && (
        <EditCategoryDialog
          category={dialog.category}
          onClose={() => setDialog({ kind: "none" })}
          onUpdate={async (categoryId, name, position) => {
            await updateCategory(categoryId, { name, position });
          }}
          onDelete={async (categoryId) => {
            await deleteCategory(categoryId);
          }}
        />
      )}
    </aside>
  );
}
