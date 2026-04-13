import { CATPPUCCIN } from "./colors";
import { CTP_COLOR_KEYS, THEME_NAMES, ThemeName } from "./types";

const THEME_STORAGE_KEY = "decentcom-theme";
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

export function loadTheme(): ThemeName {
  const raw = localStorage.getItem(THEME_STORAGE_KEY);
  if (!raw) {
    return DEFAULT_THEME;
  }
  if (!isThemeName(raw)) {
    return DEFAULT_THEME;
  }
  return raw;
}

export function saveTheme(theme: ThemeName): void {
  localStorage.setItem(THEME_STORAGE_KEY, theme);
}

export function defaultTheme(): ThemeName {
  return DEFAULT_THEME;
}
