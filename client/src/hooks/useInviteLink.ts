import { useEffect, useMemo, useState } from "react";
import { onOpenUrl } from "@tauri-apps/plugin-deep-link";

export interface ParsedInviteLink {
  serverAddress: string;
  inviteCode: string;
}

export function parseInviteLink(raw: string): ParsedInviteLink | null {
  let parsed: URL;
  try {
    parsed = new URL(raw);
  } catch {
    return null;
  }

  const parts = parsed.pathname.split("/").filter(Boolean);

  if ((parsed.protocol === "http:" || parsed.protocol === "https:") && parts[0] === "invite" && parts[1]) {
    return {
      serverAddress: parsed.origin,
      inviteCode: parts[1],
    };
  }

  if (parsed.protocol === "decentcom:") {
    const serverFromQuery = parsed.searchParams.get("server");
    if (parts[0] === "invite" && parts[1] && serverFromQuery) {
      return {
        serverAddress: serverFromQuery,
        inviteCode: parts[1],
      };
    }

    if (parsed.hostname && parts[0] === "invite" && parts[1]) {
      return {
        serverAddress: `https://${parsed.hostname}`,
        inviteCode: parts[1],
      };
    }
  }

  return null;
}

export function useInviteLink() {
  const [rawUrl, setRawUrl] = useState<string | null>(null);

  useEffect(() => {
    const current = parseInviteLink(window.location.href);
    if (current) {
      setRawUrl(window.location.href);
    }

    let disposed = false;

    void onOpenUrl((urls) => {
      if (disposed) {
        return;
      }
      const first = urls[0];
      if (!first) {
        return;
      }
      if (parseInviteLink(first)) {
        setRawUrl(first);
      }
    }).catch(() => {
      // Browser mode or plugin unavailable.
    });

    return () => {
      disposed = true;
    };
  }, []);

  const invite = useMemo(() => {
    if (!rawUrl) {
      return null;
    }
    return parseInviteLink(rawUrl);
  }, [rawUrl]);

  function clearInviteLink() {
    setRawUrl(null);
  }

  return {
    invite,
    clearInviteLink,
  };
}
