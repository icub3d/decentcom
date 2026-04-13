export const THEME_NAMES = ["latte", "frappe", "macchiato", "mocha"] as const;

export type ThemeName = (typeof THEME_NAMES)[number];

export const CTP_COLOR_KEYS = [
  "rosewater",
  "flamingo",
  "pink",
  "mauve",
  "red",
  "maroon",
  "peach",
  "yellow",
  "green",
  "teal",
  "sky",
  "sapphire",
  "blue",
  "lavender",
  "text",
  "subtext1",
  "subtext0",
  "overlay2",
  "overlay1",
  "overlay0",
  "surface2",
  "surface1",
  "surface0",
  "base",
  "mantle",
  "crust",
] as const;

export type CtpColorKey = (typeof CTP_COLOR_KEYS)[number];

export type CatppuccinPalette = Record<CtpColorKey, string>;
