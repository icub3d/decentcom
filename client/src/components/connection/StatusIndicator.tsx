type Status = "connecting" | "connected" | "disconnected";

interface StatusIndicatorProps {
  status: Status;
}

const STATUS_STYLES: Record<Status, string> = {
  connected: "bg-ctp-green/20 text-ctp-green border-ctp-green/40",
  connecting: "bg-ctp-yellow/20 text-ctp-yellow border-ctp-yellow/40",
  disconnected: "bg-ctp-red/20 text-ctp-red border-ctp-red/40",
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
