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

export class GatewayClient {
  private socket: WebSocket | null = null;
  private reconnectTimer: number | null = null;
  private shouldReconnect = true;
  private readonly getState: () => ServerStore;

  constructor(getState: () => ServerStore) {
    this.getState = getState;
  }

  connect() {
    const state = this.getState();
    if (!state.address || !state.sessionToken) {
      return;
    }

    if (this.socket && this.socket.readyState <= WebSocket.OPEN) {
      return;
    }

    const url = new URL(toGatewayUrl(state.address));
    url.searchParams.set("token", state.sessionToken);

    this.socket = new WebSocket(url.toString());

    this.socket.onopen = () => {
      this.getState().setStatus("connected");
      const channelId = this.getState().currentChannelId;
      if (channelId) {
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

    this.socket.onclose = () => {
      this.getState().setStatus("disconnected");
      this.socket = null;
      if (this.shouldReconnect) {
        this.scheduleReconnect();
      }
    };

    this.socket.onerror = () => {
      this.getState().setStatus("disconnected");
    };
  }

  disconnect() {
    this.shouldReconnect = false;
    if (this.reconnectTimer !== null) {
      window.clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    this.socket?.close();
    this.socket = null;
  }

  reconnect() {
    this.shouldReconnect = true;
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
    this.reconnectTimer = window.setTimeout(() => {
      this.reconnectTimer = null;
      this.connect();
    }, 1500);
  }
}
