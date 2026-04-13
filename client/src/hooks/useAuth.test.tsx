import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

vi.mock("../api/auth", () => ({
  authenticateWithServer: vi.fn(),
}));

import { authenticateWithServer } from "../api/auth";
import { useAuth } from "./useAuth";

describe("useAuth", () => {
  it("transitions unauthenticated -> authenticating -> authenticated", async () => {
    vi.mocked(authenticateWithServer).mockResolvedValueOnce({
      token: "tok_123",
      user_id: "user_123",
      expires_at: "2026-01-01T00:00:00Z",
    });

    const { result } = renderHook(() => useAuth());

    expect(result.current.status).toBe("unauthenticated");

    await act(async () => {
      await result.current.authenticate({
        baseUrl: "http://localhost:8080",
        pubkey: "pub",
        sign: async () => "sig",
      });
    });

    expect(result.current.status).toBe("authenticated");
    expect(result.current.token).toBe("tok_123");
    expect(result.current.userId).toBe("user_123");
  });

  it("returns to unauthenticated on failure", async () => {
    vi.mocked(authenticateWithServer).mockRejectedValueOnce(new Error("verify failed"));

    const { result } = renderHook(() => useAuth());

    await act(async () => {
      await expect(
        result.current.authenticate({
          baseUrl: "http://localhost:8080",
          pubkey: "pub",
          sign: async () => "sig",
        }),
      ).rejects.toThrow("verify failed");
    });

    expect(result.current.status).toBe("unauthenticated");
    expect(result.current.error).toContain("verify failed");
  });
});
