import { useEffect, useState } from "react";

import { useInvitesStore } from "../../stores/invites";

interface JoinByInviteProps {
  serverAddress: string;
  inviteCode: string;
  loading: boolean;
  onJoin: (serverAddress: string, inviteCode: string) => Promise<void>;
  onBack: () => void;
}

export function JoinByInvite({
  serverAddress,
  inviteCode,
  loading,
  onJoin,
  onBack,
}: JoinByInviteProps) {
  const preview = useInvitesStore((state) => state.preview);
  const fetchPreview = useInvitesStore((state) => state.getInvitePreview);
  const clearPreview = useInvitesStore((state) => state.clearPreview);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setError(null);
    void fetchPreview(serverAddress, inviteCode).catch((err) => {
      setError(err instanceof Error ? err.message : String(err));
    });
    return () => clearPreview();
  }, [clearPreview, fetchPreview, inviteCode, serverAddress]);

  return (
    <main className="min-h-screen bg-ctp-base text-ctp-text flex items-center justify-center p-6">
      <div className="w-full max-w-lg rounded-2xl border border-ctp-overlay0 bg-ctp-mantle/90 p-8 shadow-2xl space-y-4">
        <h1 className="text-2xl font-black text-ctp-blue">Join via Invite</h1>
        <p className="text-sm text-ctp-subtext1 break-all">{serverAddress}/invite/{inviteCode}</p>

        {error && (
          <p className="rounded-lg border border-ctp-red bg-ctp-red/20 p-3 text-sm text-ctp-red">
            {error}
          </p>
        )}

        {preview && (
          <div className="rounded-lg border border-ctp-overlay0 bg-ctp-base p-4 text-sm space-y-1">
            <p className="font-semibold text-ctp-text">{preview.server_name}</p>
            <p className="text-ctp-subtext0">Members: {preview.member_count}</p>
            <p className="text-ctp-subtext0">Expires: {preview.expires_at ?? "never"}</p>
          </div>
        )}

        <div className="flex gap-3">
          <button
            onClick={onBack}
            className="rounded-lg border border-ctp-overlay0 px-4 py-2 text-sm font-semibold text-ctp-subtext1"
          >
            Back
          </button>
          <button
            disabled={loading || Boolean(error)}
            onClick={() => {
              void onJoin(serverAddress, inviteCode);
            }}
            className="rounded-lg bg-ctp-blue px-4 py-2 text-sm font-bold text-ctp-crust transition hover:bg-ctp-sapphire disabled:opacity-60"
          >
            {loading ? "Joining..." : "Join Server"}
          </button>
        </div>
      </div>
    </main>
  );
}
