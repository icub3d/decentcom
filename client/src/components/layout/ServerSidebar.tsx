interface ServerSidebarProps {
  servers: Array<{ id: string; address: string }>;
  currentServerId: string | null;
  onSelectServer: (id: string) => void;
}

function initials(address: string): string {
  return new URL(address).hostname.slice(0, 2).toUpperCase();
}

export function ServerSidebar({ servers, currentServerId, onSelectServer }: ServerSidebarProps) {
  return (
    <aside className="w-20 border-r border-slate-800 bg-slate-950 p-3 flex flex-col gap-3">
      {servers.map((server) => {
        const active = server.id === currentServerId;
        return (
          <button
            key={server.id}
            onClick={() => onSelectServer(server.id)}
            title={server.address}
            className={`h-12 w-12 rounded-xl font-black text-sm transition ${
              active
                ? "bg-blue-500 text-slate-950"
                : "bg-slate-800 text-slate-200 hover:bg-slate-700"
            }`}
          >
            {initials(server.address)}
          </button>
        );
      })}
    </aside>
  );
}
