export const THEMES = [
  { id: "hud", label: "HUD", description: "Visual original do OpenHOTAS" },
  { id: "glass-cockpit", label: "Glass Cockpit", description: "Instrumentação civil dessaturada" },
] as const;

export type ThemeId = (typeof THEMES)[number]["id"];

export const DEFAULT_THEME: ThemeId = "hud";
export const THEME_STORAGE_KEY = "openhotas.theme";

export function isThemeId(value: unknown): value is ThemeId {
  return THEMES.some((theme) => theme.id === value);
}
