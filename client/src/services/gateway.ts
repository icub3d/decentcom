import type { GatewayEvent, ServerStore } from "../stores/serverStore";

interface Envelope<T = unknown> {
  op: string;
  d: T;
  t: number;
}

function toGatewayUrl(baseUrl: string): string {
  if (baseUrl.startsWith("https://")) {
    return `${baseUrl.replace("https://", "wss://")}/api/v1/gateway`;
  }
  if (baseUrl.startsWith("http://")) {
    return `${baseUrl.replace("http://", "ws://")}/api/v1/gateway`;
  }
  return `${baseUrl}/api/v1/gateway`;
}

const BASE_DELAY_MS = 1000;
const MAX_DELAY_MS = 30000;
const BACKOFF_FACTOR = 2;

export class GatewayClient {
  private socket: WebSocket | null = null;
  private reconnectTimer: number | null = null;
  private shouldReconnect = true;
  private reconnectAttempt = 0;
  private wasConnected = false;
  private readonly getState: () => ServerStore;

  constructor(getState: () => ServerStore) {
    this.getState = getState;
  }

  connect() {
    const state = this.getState();
    if (!state.address || !state.sessionToken) {
      console.log("[Gateway] connect skipped (missing address or token)");
      return;
    }

    if (this.socket && this.socket.readyState <= WebSocket.OPEN) {
      console.log("[Gateway] connect skipped (socket already open or connecting)");
      return;
    }

    console.log("[Gateway] connect start", { address: state.address, attempt: this.reconnectAttempt });

    if (this.reconnectAttempt > 0) {
      this.getState().setStatus("reconnecting");
    }

    const url = new URL(toGatewayUrl(state.address));
    url.searchParams.set("token", state.sessionToken);

    this.socket = new WebSocket(url.toString());

    this.socket.onopen = () => {
      console.log("[Gateway] socket onopen");
      const wasReconnect = this.wasConnected;
      this.reconnectAttempt = 0;
      this.wasConnected = true;
      this.getState().setStatus("connected");

      if (wasReconnect) {
        console.log("[Gateway] was reconnection, revalidating...");
        void this.getState().revalidate();
      }

      const channelId = this.getState().currentChannelId;
      if (channelId) {
        console.log("[Gateway] subscribing to initial channel", channelId);
        this.subscribe(channelId);
      }
    };

    this.socket.onmessage = (event) => {
      try {
        const envelope = JSON.parse(event.data as string) as Envelope;
        if (envelope.op === "HEARTBEAT") {
          this.send({ op: "HEARTBEAT_ACK", d: {} });
          return;
        }
        this.getState().handleGatewayEvent(envelope as GatewayEvent);
      } catch {
        // Ignore malformed gateway messages.
      }
    };

    this.socket.onclose = (event) => {
      console.log("[Gateway] socket onclose", { code: event.code, reason: event.reason, wasClean: event.wasClean });
      this.socket = null;
      if (this.shouldReconnect) {
        console.log("[Gateway] scheduling reconnect...");
        this.getState().setStatus("reconnecting");
        this.scheduleReconnect();
      } else {
        console.log("[Gateway] shouldReconnect is false, setting status to disconnected");
        this.getState().setStatus("disconnected");
      }
    };

    this.socket.onerror = (error) => {
      console.error("[Gateway] socket onerror", error);
      // onerror is always followed by onclose, so no status change needed here.
    };
  }

  disconnect() {
    this.shouldReconnect = false;
    this.reconnectAttempt = 0;
    this.wasConnected = false;
    if (this.reconnectTimer !== null) {
      window.clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    this.socket?.close();
    this.socket = null;
  }

  reconnect() {
    this.shouldReconnect = true;
    this.reconnectAttempt = 0;
    this.connect();
  }

  send(payload: Record<string, unknown>) {
    if (!this.socket || this.socket.readyState !== WebSocket.OPEN) {
      return;
    }
    this.socket.send(JSON.stringify(payload));
  }

  subscribe(channelId: string) {
    this.send({ op: "SUBSCRIBE", d: { channel_id: channelId } });
  }

  unsubscribe(channelId: string) {
    this.send({ op: "UNSUBSCRIBE", d: { channel_id: channelId } });
  }

  private scheduleReconnect() {
    if (this.reconnectTimer !== null) {
      return;
    }
    const delay = Math.min(BASE_DELAY_MS * BACKOFF_FACTOR ** this.reconnectAttempt, MAX_DELAY_MS);
    this.reconnectAttempt += 1;
    this.reconnectTimer = window.setTimeout(() => {
      this.reconnectTimer = null;
      this.connect();
    }, delay);
  }
}
