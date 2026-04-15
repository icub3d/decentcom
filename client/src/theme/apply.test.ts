import { beforeEach, describe, expect, it } from "vitest";

import { CATPPUCCIN } from "./colors";
import { applyTheme, defaultTheme } from "./apply";
import { CTP_COLOR_KEYS, THEME_NAMES } from "./types";

describe("theme apply", () => {
  beforeEach(() => {
    document.documentElement.removeAttribute("data-theme");
    for (const key of CTP_COLOR_KEYS) {
      document.documentElement.style.removeProperty(`--ctp-${key}`);
    }
    localStorage.clear();
  });

  it("applyTheme('mocha') sets mocha variables", () => {
    applyTheme("mocha");
    expect(document.documentElement.getAttribute("data-theme")).toBe("mocha");
    expect(document.documentElement.style.getPropertyValue("--ctp-base")).toBe(
      CATPPUCCIN.mocha.base,
    );
    expect(document.documentElement.style.getPropertyValue("--ctp-text")).toBe(
      CATPPUCCIN.mocha.text,
    );
  });

  it("applyTheme('latte') sets latte variables", () => {
    applyTheme("latte");
    expect(document.documentElement.getAttribute("data-theme")).toBe("latte");
    expect(document.documentElement.style.getPropertyValue("--ctp-base")).toBe(
      CATPPUCCIN.latte.base,
    );
    expect(document.documentElement.style.getPropertyValue("--ctp-blue")).toBe(
      CATPPUCCIN.latte.blue,
    );
  });

  it("all flavor maps share key sets", () => {
    const expected = Object.keys(CATPPUCCIN.mocha).sort();
    for (const name of THEME_NAMES) {
      expect(Object.keys(CATPPUCCIN[name]).sort()).toEqual(expected);
    }
  });

  it("all color values are valid hex strings", () => {
    for (const name of THEME_NAMES) {
      for (const value of Object.values(CATPPUCCIN[name])) {
        expect(value).toMatch(/^#[0-9a-fA-F]{6}$/);
      }
    }
  });

  it("defaultTheme returns mocha", () => {
    expect(defaultTheme()).toBe("mocha");
  });
});
