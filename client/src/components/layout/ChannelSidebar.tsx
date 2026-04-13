import { StatusIndicator } from "../connection/StatusIndicator";
import type { Category, Channel } from "../../stores/serverStore";

interface ChannelSidebarProps {
  channels: Channel[];
  categories: Category[];
  currentChannelId: string | null;
  status: "connecting" | "connected" | "disconnected";
  onSelectChannel: (id: string) => void;
}

export function ChannelSidebar({
  channels,
  categories,
  currentChannelId,
  status,
  onSelectChannel,
}: ChannelSidebarProps) {
  const uncategorized = channels.filter((ch) => !ch.category_id);

  return (
    <aside className="w-72 border-r border-ctp-overlay0 bg-ctp-mantle/80 p-4 overflow-auto">
      <div className="mb-4 flex items-center justify-between">
        <h2 className="text-sm font-bold uppercase tracking-wide text-ctp-subtext0">Channels</h2>
        <StatusIndicator status={status} />
      </div>

      <div className="space-y-4">
        {categories.map((category) => {
          const grouped = channels.filter((ch) => ch.category_id === category.id);
          if (!grouped.length) return null;
          return (
            <section key={category.id} className="space-y-1">
              <h3 className="text-xs font-semibold uppercase tracking-wide text-ctp-overlay1">
                {category.name}
              </h3>
              {grouped.map((channel) => (
                <button
                  key={channel.id}
                  onClick={() => onSelectChannel(channel.id)}
                  className={`w-full rounded-md px-3 py-2 text-left text-sm transition ${
                    currentChannelId === channel.id
                      ? "bg-ctp-blue/20 text-ctp-blue"
                      : "text-ctp-subtext1 hover:bg-ctp-surface0"
                  }`}
                >
                  #{channel.name}
                </button>
              ))}
            </section>
          );
        })}

        {!!uncategorized.length && (
          <section className="space-y-1">
            <h3 className="text-xs font-semibold uppercase tracking-wide text-ctp-overlay1">General</h3>
            {uncategorized.map((channel) => (
              <button
                key={channel.id}
                onClick={() => onSelectChannel(channel.id)}
                className={`w-full rounded-md px-3 py-2 text-left text-sm transition ${
                  currentChannelId === channel.id
                    ? "bg-ctp-blue/20 text-ctp-blue"
                    : "text-ctp-subtext1 hover:bg-ctp-surface0"
                }`}
              >
                #{channel.name}
              </button>
            ))}
          </section>
        )}
      </div>
    </aside>
  );
}
