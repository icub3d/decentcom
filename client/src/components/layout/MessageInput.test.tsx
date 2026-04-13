import { act, fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { MessageInput } from "./MessageInput";

describe("MessageInput", () => {
  it("sends on Enter and keeps newline on Shift+Enter", async () => {
    const onSend = vi.fn(async () => undefined);
    render(<MessageInput disabled={false} onSend={onSend} />);

    const textbox = screen.getByPlaceholderText("Send a message");
    await act(async () => {
      fireEvent.change(textbox, { target: { value: "hello" } });
      fireEvent.keyDown(textbox, { key: "Enter" });
    });

    expect(onSend).toHaveBeenCalledWith("hello");

    await act(async () => {
      fireEvent.change(textbox, { target: { value: "line1" } });
      fireEvent.keyDown(textbox, { key: "Enter", shiftKey: true });
    });

    expect(onSend).toHaveBeenCalledTimes(1);
  });

  it("is disabled when disabled=true", () => {
    const onSend = vi.fn(async () => undefined);
    render(<MessageInput disabled={true} onSend={onSend} />);

    const textbox = screen.getByPlaceholderText("Connect and select a channel to send");
    expect(textbox).toBeDisabled();
  });
});
