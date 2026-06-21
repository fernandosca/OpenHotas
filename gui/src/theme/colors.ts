import type { AxisId } from "@/types/protocol";

export const HOTAS_COLORS = {
  hud: {
    bg: "#0D0F14",
    surface: "#141720",
    surface2: "#1C2030",
    border: "rgba(255,255,255,0.06)",
    border2: "rgba(255,255,255,0.12)",
  },
  accent: {
    cyan: "#00D4FF",
    cyanDim: "rgba(0,212,255,0.12)",
    cyanGlow: "rgba(0,212,255,0.35)",
  },
  axis: {
    X: "#60A5FA",
    Y: "#34D399",
    Twist: "#F59E0B",
  } satisfies Record<AxisId, string>,
  semantic: {
    ok: "#22C55E",
    warn: "#F59E0B",
    danger: "#EF4444",
  },
  slate: {
    tick: "#94a3b8",
    dark: "#0D0F14",
  },
} as const;

export const AXIS_COLORS = HOTAS_COLORS.axis;
export const AXIS_RGB: Record<AxisId, string> = {
  X: "96,165,250",
  Y: "52,211,153",
  Twist: "245,158,11",
};

export const CANVAS_COLORS = {
  hudBackground: HOTAS_COLORS.hud.surface2,
  grid: "rgba(255,255,255,0.04)",
  centerAxis: "rgba(0,212,255,0.15)",
  subtleBorder: "rgba(255,255,255,0.08)",
  diagonalReference: "rgba(255,255,255,0.10)",
  label: "rgba(148,163,184,0.35)",
  cyanGuide: "rgba(0,212,255,0.40)",
  cyanStroke: "rgba(0,212,255,0.30)",
  cyanSelection: "rgba(0,212,255,0.10)",
  dangerFill: "rgba(239,68,68,0.05)",
  dangerGuide: "rgba(239,68,68,0.20)",
  twistRing: "rgba(245,158,11,0.12)",
  twistArc: "rgba(245,158,11,0.65)",
  twistPointer: "rgba(245,158,11,0.35)",
  markerStroke: "rgba(28,32,48,0.90)",
} as const;
