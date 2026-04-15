import { avatarUrl } from "../../api/profile";
import { useServerStore } from "../../stores/serverStore";
import { Identicon } from "./Identicon";

interface AvatarProps {
  pubkey: string;
  avatarHash: string | null | undefined;
  size?: number;
  className?: string;
}

export function Avatar({ pubkey, avatarHash, size = 32, className = "" }: AvatarProps) {
  const address = useServerStore((s) => s.address);

  if (avatarHash) {
    return (
      <img
        src={avatarUrl(address, avatarHash)}
        alt="avatar"
        width={size}
        height={size}
        className={`rounded-full object-cover ${className}`}
      />
    );
  }

  return <Identicon pubkey={pubkey} size={size} className={className} />;
}
