import { CATPPUCCIN } from "../../theme/colors";
import { THEME_NAMES, ThemeName } from "../../theme/types";

interface ThemeSwitcherProps {
  theme: ThemeName;
  onThemeSelect: (theme: ThemeName) => void;
}

export function ThemeSwitcher({ theme, onThemeSelect }: ThemeSwitcherProps) {
  return (
    <section className="rounded-xl border border-ctp-overlay0 bg-ctp-surface0 p-3 shadow-lg">
      <h3 className="mb-2 text-xs font-bold uppercase tracking-wide text-ctp-subtext0">Theme</h3>
      <div className="space-y-2">
        {THEME_NAMES.map((name) => {
          const active = theme === name;
          const palette = CATPPUCCIN[name];
          return (
            <button
              key={name}
              onClick={() => onThemeSelect(name)}
              className={`w-full rounded-lg border px-3 py-2 text-left text-sm transition ${
                active
                  ? "border-ctp-blue bg-ctp-overlay0/40 text-ctp-text"
                  : "border-ctp-overlay0 bg-ctp-base text-ctp-subtext1 hover:bg-ctp-overlay0/20"
              }`}
            >
              <div className="mb-1 flex items-center justify-between">
                <span className="font-semibold capitalize">{name}</span>
                {active && <span className="text-xs text-ctp-blue">Selected</span>}
              </div>
              <div className="flex gap-1">
                {[
                  palette.base,
                  palette.surface0,
                  palette.overlay0,
                  palette.blue,
                  palette.green,
                  palette.red,
                ].map((hex) => (
                  <span
                    key={hex}
                    className="h-3 w-3 rounded-full border border-ctp-overlay0"
                    style={{ backgroundColor: hex }}
                  />
                ))}
              </div>
            </button>
          );
        })}
      </div>
    </section>
  );
}
