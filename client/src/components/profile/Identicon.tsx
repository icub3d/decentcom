import { useMemo } from "react";
import { generateIdenticon } from "../../utils/identicon";

interface IdenticonProps {
  pubkey: string;
  size?: number;
  className?: string;
}

export function Identicon({ pubkey, size = 32, className = "" }: IdenticonProps) {
  const src = useMemo(() => generateIdenticon(pubkey, size), [pubkey, size]);

  return (
    <img
      src={src}
      alt="identicon"
      width={size}
      height={size}
      className={`rounded-full ${className}`}
    />
  );
}
