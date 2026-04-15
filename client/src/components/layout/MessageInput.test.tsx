import { act, fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { MessageInput } from "./MessageInput";

vi.mock("../../stores/serverStore", () => ({
  useServerStore: (selector: (s: Record<string, unknown>) => unknown) =>
    selector({ address: "http://localhost", sessionToken: "tok", currentChannelId: "ch1" }),
}));

describe("MessageInput", () => {
  it("sends on Enter and keeps newline on Shift+Enter", async () => {
    const onSend = vi.fn(async () => undefined);
    render(<MessageInput disabled={false} onSend={onSend} />);

    const textbox = screen.getByPlaceholderText("Send a message");
    await act(async () => {
      fireEvent.change(textbox, { target: { value: "hello" } });
      fireEvent.keyDown(textbox, { key: "Enter" });
    });

    expect(onSend).toHaveBeenCalledWith("hello", undefined);

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

  it("shows emoji button when enabled and hides when disabled", () => {
    const onSend = vi.fn(async () => undefined);
    const { rerender } = render(<MessageInput disabled={false} onSend={onSend} />);
    expect(screen.getByLabelText("Emoji picker")).toBeInTheDocument();

    rerender(<MessageInput disabled={true} onSend={onSend} />);
    expect(screen.queryByLabelText("Emoji picker")).not.toBeInTheDocument();
  });

  it("clicking emoji button opens the picker", () => {
    const onSend = vi.fn(async () => undefined);
    render(<MessageInput disabled={false} onSend={onSend} />);

    fireEvent.click(screen.getByLabelText("Emoji picker"));
    expect(screen.getByRole("dialog", { name: "Emoji picker" })).toBeInTheDocument();
  });

  it("selecting an emoji appends it to the textarea value", () => {
    const onSend = vi.fn(async () => undefined);
    render(<MessageInput disabled={false} onSend={onSend} />);

    // Type some text first
    const textbox = screen.getByPlaceholderText("Send a message");
    fireEvent.change(textbox, { target: { value: "hello " } });

    // Open picker and click an emoji
    fireEvent.click(screen.getByLabelText("Emoji picker"));
    fireEvent.click(screen.getByTitle("grinning face"));

    // Emoji should be appended (textarea value updated)
    expect(textbox).toHaveValue("hello 😀");
  });
});
