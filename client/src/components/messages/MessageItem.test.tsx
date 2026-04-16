import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { MessageItem } from "./MessageItem";

describe("MessageItem", () => {
  it("renders content, author, and edited indicator", () => {
    render(
      <MessageItem
        message={{
          id: "m1",
          channel_id: "c1",
          author_id: "u1",
          content: "hello",
          created_at: new Date().toISOString(),
          edited_at: new Date().toISOString(),
          deleted: false,
          attachments: [],
          reactions: [],
        }}
      />,
    );

    expect(screen.getByText("u1")).toBeInTheDocument();
    expect(screen.getByText("hello")).toBeInTheDocument();
    expect(screen.getByText("(edited)")).toBeInTheDocument();
  });

  it("renders deleted tombstone", () => {
    render(
      <MessageItem
        message={{
          id: "m2",
          channel_id: "c1",
          author_id: "u2",
          content: "",
          created_at: new Date().toISOString(),
          edited_at: null,
          deleted: true,
          attachments: [],
          reactions: [],
        }}
      />,
    );

    expect(screen.getByText("This message was deleted.")).toBeInTheDocument();
  });
});
