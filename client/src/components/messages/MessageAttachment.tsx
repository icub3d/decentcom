import type { Attachment } from "../../api/media";
import { mediaUrl } from "../../api/media";
import { useServerStore } from "../../stores/serverStore";

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function isImage(mime: string): boolean {
  return mime.startsWith("image/");
}

interface MessageAttachmentProps {
  attachment: Attachment;
}

export function MessageAttachment({ attachment }: MessageAttachmentProps) {
  const address = useServerStore((s) => s.address);
  const url = mediaUrl(address, attachment.content_hash);

  if (isImage(attachment.mime_type)) {
    // Constrain images to reasonable display size
    const maxW = 400;
    const maxH = 300;
    let displayW = attachment.width ?? maxW;
    let displayH = attachment.height ?? maxH;
    if (displayW > maxW) {
      displayH = Math.round(displayH * (maxW / displayW));
      displayW = maxW;
    }
    if (displayH > maxH) {
      displayW = Math.round(displayW * (maxH / displayH));
      displayH = maxH;
    }

    return (
      <a href={url} target="_blank" rel="noopener noreferrer" className="block mt-1">
        <img
          src={url}
          alt={attachment.filename}
          width={displayW}
          height={displayH}
          className="max-w-full rounded-lg border border-ctp-overlay0"
          loading="lazy"
        />
      </a>
    );
  }

  return (
    <a
      href={url}
      target="_blank"
      rel="noopener noreferrer"
      className="mt-1 flex items-center gap-2 rounded-lg border border-ctp-overlay0 bg-ctp-surface0/50 px-3 py-2 text-sm text-ctp-blue hover:bg-ctp-surface0 transition max-w-xs"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
        className="flex-shrink-0"
      >
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
        <polyline points="14 2 14 8 20 8" />
      </svg>
      <span className="truncate">{attachment.filename}</span>
      <span className="flex-shrink-0 text-ctp-subtext0">{formatSize(attachment.size)}</span>
    </a>
  );
}
