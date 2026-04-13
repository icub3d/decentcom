# Feature: Theming

## Overview
Theming integrates the Catppuccin color palette into the client application, with Mocha as the default dark theme. Users can switch between all four Catppuccin flavors (Latte, Frappe, Macchiato, Mocha) and their preference is persisted locally. All UI components use theme tokens via Tailwind CSS custom properties, so switching themes is instant and consistent.

## Background
The project stack specifies Tailwind CSS with Catppuccin themes and Mocha as the default (`CLAUDE.md`). Catppuccin provides four flavors: Latte (light), Frappe (medium dark), Macchiato (dark), and Mocha (darkest). Each flavor defines a consistent set of named colors (base, mantle, crust, text, subtext, overlay, surface, and accent colors like rosewater, flamingo, pink, mauve, red, maroon, peach, yellow, green, teal, sapphire, blue, lavender). This feature depends on the client shell (feature 9) for the UI structure.

## Requirements
- [ ] All UI components use Catppuccin color tokens via CSS custom properties — no hardcoded color values.
- [ ] Mocha is the default theme applied on first launch.
- [ ] Users can switch between Latte, Frappe, Macchiato, and Mocha via a theme selector in the UI.
- [ ] Theme changes apply instantly without a page reload.
- [ ] The selected theme is persisted in local storage and restored on app restart.
- [ ] The Tailwind v4 `@theme` block maps `ctp-*` names to CSS custom properties.
- [ ] The theme works correctly with all existing components (server sidebar, channel sidebar, message view, message input).

## Design

### API / Interface Changes

No server-side changes. This is entirely a client-side feature.

**Tauri IPC (optional):**
Theme preference could be stored via Tauri's local storage API or browser `localStorage`. Browser `localStorage` is simpler and sufficient since theme preference is not sensitive data.

### Data Model Changes

No database changes. Client-side only:

- Theme preference stored in `localStorage` under key `decentcom-theme`.
- The `AppStore` Zustand store holds the current theme in memory.

### Component Changes

**New files (client, `client/src/`):**

- `client/src/theme/colors.ts` -- Catppuccin color definitions for all four flavors, exported as typed objects.
- `client/src/theme/apply.ts` -- `applyTheme(flavor: ThemeName)` function that sets CSS custom properties on `document.documentElement`.
- `client/src/theme/types.ts` -- `ThemeName` type (`"latte" | "frappe" | "macchiato" | "mocha"`), color token type definitions.
- `client/src/components/settings/ThemeSwitcher.tsx` -- UI component showing the four theme options with preview swatches, click to switch.

**Modified files:**

- `client/tailwind.config.ts` -- Extend the Tailwind theme to use CSS custom properties for all Catppuccin color tokens (e.g., `bg-base`, `text-text`, `bg-surface0`, `text-subtext1`, `accent-blue`).
- `client/src/index.css` (or global CSS) -- Define the CSS custom property defaults (Mocha values) and ensure the theme variables are set on `:root`.
- `client/src/stores/appStore.ts` -- Add `theme` field and `setTheme(name: ThemeName)` action. On theme change, call `applyTheme()` and persist to `localStorage`.
- `client/src/App.tsx` -- On mount, read theme from `localStorage` (default to `"mocha"`), call `applyTheme()`.
- All existing components -- Replace any hardcoded Tailwind color classes (e.g., `bg-gray-900`) with theme token classes (e.g., `bg-base`, `bg-mantle`, `text-text`).

**Catppuccin color token mapping for Tailwind:**

```
base       -> Main background
mantle     -> Sidebar/panel background
crust      -> Deepest background (e.g., server sidebar)
text       -> Primary text
subtext0   -> Muted text
subtext1   -> Secondary text
overlay0   -> Borders, dividers
overlay1   -> Hover states
overlay2   -> Active states
surface0   -> Card/input backgrounds
surface1   -> Elevated surfaces
surface2   -> Higher elevation
blue       -> Primary accent (links, active items)
green      -> Success states
red        -> Error/destructive states
yellow     -> Warning states
peach      -> Secondary accent
mauve      -> Tertiary accent
```

## Task List

### Phase A: Tailwind and CSS setup
- [ ] Install `@catppuccin/palette` npm package.
- [ ] `client/src/theme/colors.ts` — all four flavor palettes extracted from `@catppuccin/palette`.
- [ ] `client/src/theme/types.ts` — `ThemeName` and `CatppuccinPalette` types.
- [ ] `client/src/theme/apply.ts` — `applyTheme()`, `loadTheme()`, `saveTheme()`.
- [ ] Tailwind v4 `@theme` block in `App.css` maps `--color-ctp-*` to CSS var references.
- [ ] Default Mocha `:root` values in `App.css`.

### Phase B: State and persistence
- [ ] `client/src/stores/appStore.ts` — `theme` state, `setTheme()`, `initTheme()`.
- [ ] `App.tsx` calls `initTheme()` on mount via `useEffect`.

### Phase C: Theme switcher UI
- [ ] `client/src/components/settings/ThemeSwitcher.tsx` — four options with color swatches, active checkmark.
- [ ] Gear button in `ServerSidebar` toggles theme panel.

### Phase D: Migrate existing components
- [ ] All components were authored with `ctp-*` token classes from the start; no migration needed.

## Test List
- [ ] Unit test: `applyTheme("mocha")` sets the correct CSS custom properties on the document root.
- [ ] Unit test: `applyTheme("latte")` sets Latte-specific values.
- [ ] Unit test: All four flavor color maps have the same set of keys.
- [ ] Unit test: All color values are valid hex strings.
- [ ] Unit test: `loadTheme()` defaults to mocha; `saveTheme()` persists all valid flavors.
- [ ] Unit test: `loadTheme()` ignores unknown stored values and defaults to mocha.
- [ ] Unit test: `ThemeSwitcher` renders four options and calls `setTheme` on click. (Deferred — requires mock for appStore.)
- [ ] Manual: Launch the app, verify Mocha theme is applied by default.
- [ ] Manual: Open the theme switcher, select each flavor, verify colors change instantly.
- [ ] Manual: Select Latte, restart the app, verify Latte is still applied.

## Open Questions
- Should the app support system theme detection (follow OS light/dark preference) as an "auto" option? This is a nice UX touch but adds complexity. Could be a fast follow-up.
- Should accent colors be user-customizable beyond the four Catppuccin flavors? Catppuccin's palette is well-designed, so custom themes may not be needed initially.
- Where should the theme switcher live in the UI? Options: settings panel, dropdown from user avatar, or a dedicated button in the sidebar. A gear icon in the bottom of the server sidebar is a common pattern.
