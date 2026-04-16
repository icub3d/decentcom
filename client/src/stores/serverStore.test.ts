import { describe, expect, it, vi, beforeEach } from "vitest";

// Mock Tauri and service dependencies before importing the store.
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("../services/api", () => ({
  ApiError: class ApiError extends Error {},
  apiRequest: vi.fn(),
}));
vi.mock("../services/auth", () => ({ authenticateServer: vi.fn() }));
vi.mock("../services/gateway", () => ({
  GatewayClient: vi.fn().mockImplementation(() => ({
    connect: vi.fn(),
    disconnect: vi.fn(),
  })),
}));
vi.mock("../api/channels", () => ({
  listChannels: vi.fn(),
  createChannel: vi.fn(),
  updateChannel: vi.fn(),
  deleteChannel: vi.fn(),
}));
vi.mock("../api/roles", () => ({ listRoles: vi.fn() }));
vi.mock("../api/members", () => ({ joinServer: vi.fn() }));
vi.mock("./members", () => ({ useMembersStore: { getState: vi.fn(() => ({ reset: vi.fn() })) } }));

import { apiRequest } from "../services/api";
import { useServerStore } from "./serverStore";
import type { Message } from "./serverStore";

const mockApiRequest = vi.mocked(apiRequest);

function makeMsg(id: string, channelId = "ch1"): Message {
  return {
    id,
    channel_id: channelId,
    author_id: "user1",
    content: `msg ${id}`,
    created_at: id,
    edited_at: null,
    deleted: false,
    attachments: [],
  };
}

describe("serverStore — message ordering & deduplication", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useServerStore.setState({
      messages: {},
      hasMore: {},
      sessionToken: "tok",
      address: "http://localhost",
      serverId: "",
      status: "disconnected",
      channels: [],
      currentChannelId: null,
      roles: [],
    } as Parameters<typeof useServerStore.setState>[0]);
  });

  it("stale-write race: WS message arriving during in-flight fetch survives in correct position", async () => {
    // Oldest message already in store (already loaded).
    const old = makeMsg("msg-001");
    useServerStore.setState({ messages: { ch1: [old] } });

    // The API will return a page with one message.
    const fetched = makeMsg("msg-000");
    let resolveApi!: (v: unknown) => void;
    mockApiRequest.mockReturnValueOnce(
      new Promise((res) => {
        resolveApi = res;
      }),
    );

    const loadPromise = useServerStore.getState().loadMoreMessages("ch1");

    // Simulate WS MESSAGE_CREATE arriving while fetch is in flight.
    const ws = makeMsg("msg-002");
    useServerStore.getState().handleGatewayEvent({ op: "MESSAGE_CREATE", d: ws, t: 1 });

    // Now resolve the API call.
    resolveApi({ messages: [fetched], has_more: false });
    await loadPromise;

    const msgs = useServerStore.getState().messages["ch1"];
    const ids = msgs.map((m) => m.id);
    // Newest-first order: msg-002, msg-001, msg-000
    expect(ids).toEqual(["msg-002", "msg-001", "msg-000"]);
  });

  it("concurrent pagination calls produce no duplicates", async () => {
    const existing = makeMsg("msg-010");
    useServerStore.setState({ messages: { ch1: [existing] } });

    const page1 = { messages: [makeMsg("msg-009"), makeMsg("msg-008")], has_more: false };
    const page2 = { messages: [makeMsg("msg-009"), makeMsg("msg-008")], has_more: false };

    let resolve1!: (v: unknown) => void;
    let resolve2!: (v: unknown) => void;
    mockApiRequest
      .mockReturnValueOnce(new Promise((r) => { resolve1 = r; }))
      .mockReturnValueOnce(new Promise((r) => { resolve2 = r; }));

    const p1 = useServerStore.getState().loadMoreMessages("ch1");
    const p2 = useServerStore.getState().loadMoreMessages("ch1");

    resolve1(page1);
    resolve2(page2);
    await Promise.all([p1, p2]);

    const msgs = useServerStore.getState().messages["ch1"];
    const ids = msgs.map((m) => m.id);
    // Each ID appears exactly once.
    expect(ids).toEqual([...new Set(ids)]);
    expect(ids).toContain("msg-010");
    expect(ids).toContain("msg-009");
    expect(ids).toContain("msg-008");
    expect(ids.length).toBe(3);
  });

  it("WS backlog (out-of-order arrivals) is sorted correctly", () => {
    // Seed store with some messages.
    useServerStore.setState({
      messages: {
        ch1: [makeMsg("msg-010"), makeMsg("msg-009"), makeMsg("msg-008")],
      },
    });

    // Backlog messages arrive out of order via WS.
    useServerStore.getState().handleGatewayEvent({ op: "MESSAGE_CREATE", d: makeMsg("msg-007"), t: 1 });
    useServerStore.getState().handleGatewayEvent({ op: "MESSAGE_CREATE", d: makeMsg("msg-012"), t: 2 });
    useServerStore.getState().handleGatewayEvent({ op: "MESSAGE_CREATE", d: makeMsg("msg-011"), t: 3 });

    const msgs = useServerStore.getState().messages["ch1"];
    const ids = msgs.map((m) => m.id);
    expect(ids).toEqual(["msg-012", "msg-011", "msg-010", "msg-009", "msg-008", "msg-007"]);
  });

  it("WS duplicate is ignored", () => {
    const msg = makeMsg("msg-001");
    useServerStore.setState({ messages: { ch1: [msg] } });

    useServerStore.getState().handleGatewayEvent({ op: "MESSAGE_CREATE", d: { ...msg }, t: 1 });

    expect(useServerStore.getState().messages["ch1"].length).toBe(1);
  });
});
