import { useEffect, useState } from "react";
import { getMemberProfile, type MemberProfile } from "../../api/profile";
import { useServerStore } from "../../stores/serverStore";
import { Avatar } from "./Avatar";

interface ProfilePopoverProps {
  pubkey: string;
  onClose: () => void;
}

function truncatePubkey(pubkey: string): string {
  if (pubkey.length <= 20) {
    return pubkey;
  }
  return `${pubkey.slice(0, 10)}...${pubkey.slice(-8)}`;
}

export function ProfilePopover({ pubkey, onClose }: ProfilePopoverProps) {
  const address = useServerStore((s) => s.address);
  const token = useServerStore((s) => s.sessionToken);
  const [profile, setProfile] = useState<MemberProfile | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!token) return;
    getMemberProfile(address, token, pubkey)
      .then(setProfile)
      .catch((err) => setError(err instanceof Error ? err.message : String(err)));
  }, [address, token, pubkey]);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-ctp-crust/60" onClick={onClose}>
      <div
        className="w-72 rounded-xl border border-ctp-overlay0 bg-ctp-mantle p-4 shadow-lg"
        onClick={(e) => e.stopPropagation()}
      >
        {error ? (
          <p className="text-sm text-ctp-red">{error}</p>
        ) : !profile ? (
          <p className="text-sm text-ctp-subtext0">Loading…</p>
        ) : (
          <>
            <div className="mb-3 flex items-center gap-3">
              <Avatar pubkey={profile.pubkey} avatarHash={profile.avatar_hash} size={48} />
              <div className="min-w-0">
                <div className="truncate text-sm font-bold text-ctp-text">
                  {profile.display_name ?? truncatePubkey(profile.pubkey)}
                </div>
                <div className="truncate text-xs text-ctp-subtext0">
                  {truncatePubkey(profile.pubkey)}
                </div>
              </div>
            </div>
            <div className="mb-2 text-xs text-ctp-subtext0">
              Joined {new Date(profile.joined_at).toLocaleDateString()}
            </div>
            {profile.roles.length > 0 && (
              <div className="flex flex-wrap gap-1">
                {profile.roles.map((role) => (
                  <span
                    key={role.id}
                    className="rounded-md border border-ctp-overlay0 bg-ctp-surface0 px-2 py-0.5 text-xs text-ctp-subtext1"
                  >
                    {role.name}
                  </span>
                ))}
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}
