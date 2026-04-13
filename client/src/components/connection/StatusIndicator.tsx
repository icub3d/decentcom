type Status = "connecting" | "connected" | "disconnected";

interface StatusIndicatorProps {
  status: Status;
}

const STATUS_STYLES: Record<Status, string> = {
  connected: "bg-emerald-500/20 text-emerald-300 border-emerald-500/40",
  connecting: "bg-amber-500/20 text-amber-300 border-amber-500/40",
  disconnected: "bg-rose-500/20 text-rose-300 border-rose-500/40",
};

export function StatusIndicator({ status }: StatusIndicatorProps) {
  return (
    <span
      className={`inline-flex items-center rounded-full border px-2 py-1 text-xs font-semibold uppercase tracking-wide ${STATUS_STYLES[status]}`}
    >
      {status}
    </span>
  );
}
