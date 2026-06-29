import type { AxisId } from "@/types/protocol";

type ThemeColorName =
  | "surface" | "grid" | "center" | "reference" | "label" | "marker"
  | "accent" | "accentGlow" | "axisX" | "axisY" | "axisTwist"
  | "danger" | "textPrimary" | "textDim";

const THEME_COLOR_VARS: Record<ThemeColorName, string> = {
  surface: "--theme-surface",
  grid: "--theme-canvas-grid",
  center: "--theme-canvas-center",
  reference: "--theme-canvas-reference",
  label: "--theme-canvas-label",
  marker: "--theme-canvas-marker",
  accent: "--theme-accent",
  accentGlow: "--theme-accent-glow",
  axisX: "--theme-axis-x",
  axisY: "--theme-axis-y",
  axisTwist: "--theme-axis-twist",
  danger: "--theme-danger",
  textPrimary: "--theme-text-primary",
  textDim: "--theme-text-dim",
};

export function getThemeColor(name: ThemeColorName): string {
  return getComputedStyle(document.documentElement)
    .getPropertyValue(THEME_COLOR_VARS[name])
    .trim();
}

export const AXIS_COLORS: Record<AxisId, string> = {
  X: "var(--theme-axis-x)",
  Y: "var(--theme-axis-y)",
  Twist: "var(--theme-axis-twist)",
};

export const AXIS_RGB: Record<AxisId, string> = {
  X: "var(--theme-axis-x-rgb)",
  Y: "var(--theme-axis-y-rgb)",
  Twist: "var(--theme-axis-twist-rgb)",
};
