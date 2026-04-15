import { CATPPUCCIN } from "./colors";
import { CTP_COLOR_KEYS, THEME_NAMES, ThemeName } from "./types";

const DEFAULT_THEME: ThemeName = "mocha";

export function applyTheme(theme: ThemeName): void {
  const palette = CATPPUCCIN[theme];
  const root = document.documentElement;

  for (const key of CTP_COLOR_KEYS) {
    root.style.setProperty(`--ctp-${key}`, palette[key]);
  }

  root.setAttribute("data-theme", theme);
}

export function isThemeName(value: string): value is ThemeName {
  return THEME_NAMES.includes(value as ThemeName);
}

export function defaultTheme(): ThemeName {
  return DEFAULT_THEME;
}
