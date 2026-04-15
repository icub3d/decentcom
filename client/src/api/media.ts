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
): Promise<Attachment> {
  const formData = new FormData();
  formData.append("file", file);

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

  const data = (await response.json()) as Attachment[];
  if (!data.length) {
    throw new Error("upload returned no attachments");
  }
  return data[0];
}

export function mediaUrl(baseUrl: string, hash: string): string {
  return `${baseUrl}/api/v1/media/${hash}`;
}
