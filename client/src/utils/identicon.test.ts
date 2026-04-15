import { describe, it, expect } from "vitest";
import { generateIdenticon } from "./identicon";

describe("generateIdenticon", () => {
  it("returns a data URI", () => {
    const result = generateIdenticon("abcdef1234567890abcdef1234567890");
    expect(result).toMatch(/^data:image\/svg\+xml;base64,/);
  });

  it("is deterministic for the same pubkey", () => {
    const pubkey = "aabbccdd11223344aabbccdd11223344";
    const a = generateIdenticon(pubkey);
    const b = generateIdenticon(pubkey);
    expect(a).toBe(b);
  });

  it("produces different output for different pubkeys", () => {
    const a = generateIdenticon("0000000000000000000000000000000000000000");
    const b = generateIdenticon("ffffffffffffffffffffffffffffffffffffffff");
    expect(a).not.toBe(b);
  });
});
