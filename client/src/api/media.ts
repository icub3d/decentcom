export interface Attachment {
  id: string;
  message_id: string | null;
  channel_id: string;
  uploader_id: string;
  filename: string;
  content_hash: string;
  size: number;
  mime_type: string;
  width: number | null;
  height: number | null;
  created_at: string;
}

export async function uploadFile(
  baseUrl: string,
  token: string,
  channelId: string,
  file: File,
  onProgress?: (pct: number) => void,
): Promise<Attachment> {
  const formData = new FormData();
  formData.append("file", file);

  // XMLHttpRequest for progress tracking
  if (onProgress) {
    return new Promise<Attachment>((resolve, reject) => {
      const xhr = new XMLHttpRequest();
      xhr.open("POST", `${baseUrl}/api/v1/channels/${channelId}/attachments`);
      xhr.setRequestHeader("Authorization", `Bearer ${token}`);

      xhr.upload.addEventListener("progress", (e) => {
        if (e.lengthComputable) {
          onProgress(Math.round((e.loaded / e.total) * 100));
        }
      });

      xhr.addEventListener("load", () => {
        if (xhr.status >= 200 && xhr.status < 300) {
          resolve(JSON.parse(xhr.responseText) as Attachment);
        } else {
          let message = `upload failed (${xhr.status})`;
          try {
            const body = JSON.parse(xhr.responseText) as { error?: string };
            if (body?.error) message = body.error;
          } catch {
            // ignore
          }
          reject(new Error(message));
        }
      });

      xhr.addEventListener("error", () => reject(new Error("upload failed")));
      xhr.send(formData);
    });
  }

  // Simple fetch when no progress needed
  const response = await fetch(
    `${baseUrl}/api/v1/channels/${channelId}/attachments`,
    {
      method: "POST",
      headers: { Authorization: `Bearer ${token}` },
      body: formData,
    },
  );

  if (!response.ok) {
    let message = `upload failed (${response.status})`;
    try {
      const body = (await response.json()) as { error?: string };
      if (body?.error) message = body.error;
    } catch {
      // ignore
    }
    throw new Error(message);
  }

  return (await response.json()) as Attachment;
}

export function mediaUrl(baseUrl: string, hash: string): string {
  return `${baseUrl}/api/v1/media/${hash}`;
}
