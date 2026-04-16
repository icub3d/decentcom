import { fireEvent, render, screen } from "@testing-library/react";
import { createRef } from "react";
import { describe, expect, it, vi } from "vitest";

import { EmojiPicker } from "./EmojiPicker";
import { EMOJI_CATEGORIES } from "../../data/emojis";

const nullRef = createRef<HTMLElement>();

describe("EmojiPicker", () => {
  it("renders category tabs and emoji buttons", () => {
    render(<EmojiPicker anchorRef={nullRef} onSelect={vi.fn()} onClose={vi.fn()} />);
    // All 8 category tabs
    const tabs = screen.getAllByRole("tab");
    expect(tabs).toHaveLength(EMOJI_CATEGORIES.length);
    // Emoji buttons exist in the grid
    const emojiButtons = screen.getAllByTitle(/.+/);
    expect(emojiButtons.length).toBeGreaterThan(0);
  });

  it("clicking an emoji calls onSelect with the correct string", () => {
    const onSelect = vi.fn();
    render(<EmojiPicker anchorRef={nullRef} onSelect={onSelect} onClose={vi.fn()} />);
    // Click the first emoji (grinning face)
    const firstEmoji = screen.getByTitle("grinning face");
    fireEvent.click(firstEmoji);
    expect(onSelect).toHaveBeenCalledWith("😀");
  });

  it("pressing Escape calls onClose", () => {
    const onClose = vi.fn();
    render(<EmojiPicker anchorRef={nullRef} onSelect={vi.fn()} onClose={onClose} />);
    fireEvent.keyDown(document, { key: "Escape" });
    expect(onClose).toHaveBeenCalled();
  });

  it("outside mousedown calls onClose", () => {
    const onClose = vi.fn();
    render(<EmojiPicker anchorRef={nullRef} onSelect={vi.fn()} onClose={onClose} />);
    fireEvent.mouseDown(document.body);
    expect(onClose).toHaveBeenCalled();
  });

  it("category tab filters the visible emoji grid", () => {
    render(<EmojiPicker anchorRef={nullRef} onSelect={vi.fn()} onClose={vi.fn()} />);
    // Default is "Smileys & Emotion" — should have grinning face
    expect(screen.getByTitle("grinning face")).toBeInTheDocument();
    expect(screen.queryByTitle("waving hand")).not.toBeInTheDocument();

    // Click "People & Body" tab
    const peopleTab = screen.getByRole("tab", { name: "People & Body" });
    fireEvent.click(peopleTab);
    expect(screen.getByTitle("waving hand")).toBeInTheDocument();
    expect(screen.queryByTitle("grinning face")).not.toBeInTheDocument();
  });
});
