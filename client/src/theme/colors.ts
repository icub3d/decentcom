import { flavors } from "@catppuccin/palette";

import { CTP_COLOR_KEYS, CatppuccinPalette, ThemeName } from "./types";

type RawFlavor = {
  colors: Record<string, { hex: string }>;
};

function toPalette(flavor: RawFlavor): CatppuccinPalette {
  const entries = CTP_COLOR_KEYS.map((key) => [key, flavor.colors[key].hex]);
  return Object.fromEntries(entries) as CatppuccinPalette;
}

export const CATPPUCCIN: Record<ThemeName, CatppuccinPalette> = {
  latte: toPalette(flavors.latte),
  frappe: toPalette(flavors.frappe),
  macchiato: toPalette(flavors.macchiato),
  mocha: toPalette(flavors.mocha),
};
