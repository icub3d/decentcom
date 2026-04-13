import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { SeedPhrase } from "./SeedPhrase";

describe("SeedPhrase", () => {
  it("displays all 24 words with positional numbers", () => {
    const words = Array.from({ length: 24 }, (_, i) => `word${i + 1}`);
    render(<SeedPhrase seedPhrase={words} onConfirm={vi.fn()} />);

    for (let i = 0; i < 24; i++) {
      expect(screen.getByText(`word${i + 1}`)).toBeInTheDocument();
      expect(screen.getByText(`${i + 1}.`)).toBeInTheDocument();
    }
  });
});
