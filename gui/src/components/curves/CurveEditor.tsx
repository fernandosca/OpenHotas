import { useCallback, useEffect, useRef } from "react";
import { cn } from "@/lib/utils";
import { getThemeColor } from "@/theme/colors";
import { useTheme } from "@/theme/ThemeProvider";
import type { ResponseCurveData } from "@/types/protocol";

type Point = [number, number];

interface Props {
  axisIndex: 0 | 1 | 2;
  responseCurve: ResponseCurveData;
  deadzonePermille: number;
  disabled?: boolean;
}

const PAD = 36;
const CANVAS_HEIGHT = 220;

function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
}

function applyDeadzone(input: number, threshold: number): number {
  const value = clamp(input, -1, 1);
  const abs = Math.abs(value);
  if (abs < threshold) return 0;

  const sign = value >= 0 ? 1 : -1;
  return sign * ((abs - threshold) / (1 - threshold));
}

function piecewiseLinear(input: number, curve: ResponseCurveData): number {
  const x = clamp(input, -1, 1);

  const p0: Point = [-1, -1];
  const p1: Point = [curve.point_left.x / 1000, curve.point_left.y / 1000];
  const p2: Point = [0, 0];
  const p3: Point = [curve.point_right.x / 1000, curve.point_right.y / 1000];
  const p4: Point = [1, 1];

  const points: Point[] = [p0, p1, p2, p3, p4];

  for (let i = 0; i < 4; i++) {
    const [x0, y0] = points[i];
    const [x1, y1] = points[i + 1];
    if (x <= x1) {
      const dx = x1 - x0;
      if (Math.abs(dx) < 1e-9) return clamp(y0, -1, 1);
      const t = (x - x0) / dx;
      return clamp(y0 + t * (y1 - y0), -1, 1);
    }
  }

  return 1;
}

function buildResponseCurve(
  responseCurve: ResponseCurveData,
  deadzonePermille: number,
  steps = 256,
): Point[] {
  const deadzone = clamp(deadzonePermille / 1000, 0, 1);

  return Array.from({ length: steps + 1 }, (_, index) => {
    const nx = index / steps;
    const input = nx * 2 - 1;
    const dz = applyDeadzone(input, deadzone);
    const out = piecewiseLinear(dz, responseCurve);
    return [nx, (clamp(out, -1, 1) + 1) / 2] as Point;
  });
}

export function CurveEditor({ axisIndex, responseCurve, deadzonePermille, disabled = false }: Props) {
  const ref = useRef<HTMLCanvasElement>(null);
  const { theme } = useTheme();

  function dims() {
    const c = ref.current;
    if (!c) return { W: 0, H: 0, PW: 0, PH: 0 };
    const rect = c.getBoundingClientRect();
    const W = Math.round(rect.width || 420);
    const H = Math.round(rect.height || CANVAS_HEIGHT);
    return { W, H, PW: W - PAD * 2, PH: H - PAD * 2 };
  }

  function toC(nx: number, ny: number): [number, number] {
    const { PW, PH } = dims();
    return [PAD + nx * PW, PAD + (1 - ny) * PH];
  }

  const draw = useCallback(() => {
    const canvas = ref.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const { W, H, PW, PH } = dims();
    const dpr = window.devicePixelRatio || 1;
    const nextWidth = Math.max(1, Math.round(W * dpr));
    const nextHeight = Math.max(1, Math.round(H * dpr));
    if (canvas.width !== nextWidth || canvas.height !== nextHeight) {
      canvas.width = nextWidth;
      canvas.height = nextHeight;
    }
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

    const curve = buildResponseCurve(responseCurve, deadzonePermille);
    const deadzone = clamp(deadzonePermille / 1000, 0, 1);
    const color = getThemeColor((["axisX", "axisY", "axisTwist"] as const)[axisIndex]);
    const colors = {
      surface: getThemeColor("surface"), grid: getThemeColor("grid"),
      reference: getThemeColor("reference"), center: getThemeColor("center"),
      label: getThemeColor("label"), danger: getThemeColor("danger"),
      marker: getThemeColor("textPrimary"), disabled: getThemeColor("textDim"),
    };

    ctx.clearRect(0, 0, W, H);

    ctx.fillStyle = colors.surface;
    ctx.beginPath();
    if (ctx.roundRect) ctx.roundRect(0, 0, W, H, 8);
    else ctx.rect(0, 0, W, H);
    ctx.fill();

    ctx.strokeStyle = colors.grid;
    ctx.lineWidth = 0.5;
    for (let i = 0; i <= 4; i++) {
      ctx.beginPath();
      ctx.moveTo(PAD + i * PW / 4, PAD);
      ctx.lineTo(PAD + i * PW / 4, PAD + PH);
      ctx.stroke();

      ctx.beginPath();
      ctx.moveTo(PAD, PAD + i * PH / 4);
      ctx.lineTo(PAD + PW, PAD + i * PH / 4);
      ctx.stroke();
    }

    ctx.strokeStyle = colors.reference;
    ctx.lineWidth = 0.5;
    ctx.setLineDash([4, 4]);
    ctx.beginPath();
    ctx.moveTo(PAD, PAD + PH);
    ctx.lineTo(PAD + PW, PAD);
    ctx.stroke();
    ctx.setLineDash([]);

    if (deadzone > 0) {
      const center = PAD + PW / 2;
      const halfWidth = deadzone * PW / 2;
      ctx.fillStyle = colors.danger; ctx.globalAlpha = 0.05;
      ctx.fillRect(center - halfWidth, PAD, halfWidth * 2, PH);

      ctx.globalAlpha = 0.2; ctx.strokeStyle = colors.danger;
      ctx.lineWidth = 0.5;
      ctx.setLineDash([3, 2]);
      ctx.beginPath();
      ctx.moveTo(center - halfWidth, PAD);
      ctx.lineTo(center - halfWidth, PAD + PH);
      ctx.stroke();
      ctx.beginPath();
      ctx.moveTo(center + halfWidth, PAD);
      ctx.lineTo(center + halfWidth, PAD + PH);
      ctx.stroke();
      ctx.setLineDash([]);
      ctx.globalAlpha = 1;
    }

    ctx.strokeStyle = colors.center;
    ctx.lineWidth = 0.75;
    ctx.beginPath();
    ctx.moveTo(PAD, PAD + PH / 2);
    ctx.lineTo(PAD + PW, PAD + PH / 2);
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(PAD + PW / 2, PAD);
    ctx.lineTo(PAD + PW / 2, PAD + PH);
    ctx.stroke();

    ctx.strokeStyle = colors.reference;
    ctx.lineWidth = 1;
    ctx.strokeRect(PAD, PAD, PW, PH);

    ctx.fillStyle = colors.label;
    ctx.font = "10px monospace";
    ctx.textAlign = "center";
    for (let i = 0; i <= 4; i++) {
      ctx.fillText(`${i * 25}%`, Math.round(PAD + i * PW / 4), Math.round(PAD + PH + 13));
    }
    ctx.textAlign = "right";
    for (let i = 0; i <= 4; i++) {
      ctx.fillText(`${i * 25}%`, PAD - 5, Math.round(PAD + PH - i * PH / 4 + 3));
    }

    const h2r = (hex: string, alpha: number) => {
      const r = parseInt(hex.slice(1, 3), 16);
      const g = parseInt(hex.slice(3, 5), 16);
      const b = parseInt(hex.slice(5, 7), 16);
      return `rgba(${r},${g},${b},${alpha})`;
    };

    ctx.beginPath();
    curve.forEach(([nx, ny], index) => {
      const [cx, cy] = toC(nx, ny);
      if (index === 0) ctx.moveTo(cx, cy);
      else ctx.lineTo(cx, cy);
    });
    ctx.strokeStyle = disabled ? colors.label : color;
    ctx.lineWidth = 2;
    ctx.stroke();

    ctx.beginPath();
    curve.forEach(([nx, ny], index) => {
      const [cx, cy] = toC(nx, ny);
      if (index === 0) ctx.moveTo(cx, cy);
      else ctx.lineTo(cx, cy);
    });
    ctx.lineTo(PAD + PW, PAD + PH);
    ctx.lineTo(PAD, PAD + PH);
    ctx.closePath();
    ctx.fillStyle = h2r(disabled ? colors.disabled : color, 0.07);
    ctx.fill();

    // Draw control points P1 and P3
    if (!disabled) {
      const controlPoints = [
        { pt: responseCurve.point_left, label: "P1" },
        { pt: responseCurve.point_right, label: "P3" },
      ];

      for (const { pt } of controlPoints) {
        const nx = (pt.x / 1000 + 1) / 2;
        const ny = (pt.y / 1000 + 1) / 2;
        const [cx, cy] = toC(nx, ny);

        ctx.beginPath();
        ctx.arc(cx, cy, 5, 0, Math.PI * 2);
        ctx.fillStyle = color;
        ctx.fill();
        ctx.strokeStyle = colors.marker;
        ctx.lineWidth = 1.5;
        ctx.stroke();
      }
    }
  }, [axisIndex, disabled, responseCurve, deadzonePermille, theme]);

  useEffect(() => {
    draw();
    window.addEventListener("resize", draw);
    return () => window.removeEventListener("resize", draw);
  }, [draw]);

  return (
    <div className={cn("space-y-3", disabled && "opacity-55")}>
      <canvas
        ref={ref}
        style={{ width: "100%", height: CANVAS_HEIGHT, display: "block", cursor: "default" }}
        aria-label="Visualização da curva de resposta"
      />

      <div className="flex items-center justify-between gap-3 text-[10px] font-mono text-content-muted">
        <span>
          P1 ({responseCurve.point_left.x}, {responseCurve.point_left.y})
        </span>
        <span>deadzone {(deadzonePermille / 10).toFixed(1)}%</span>
        <span>
          P3 ({responseCurve.point_right.x}, {responseCurve.point_right.y})
        </span>
      </div>
    </div>
  );
}

export type { Point };
