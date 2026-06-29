import { useRef, useEffect } from "react";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Slider } from "@/components/ui/slider";
import { Switch } from "@/components/ui/switch";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import type { UseDeviceConfigReturn } from "@/hooks/useDeviceConfig";
import type { DeviceSnapshot } from "@/hooks/useDevicePolling";
import type { AxisId, AxisToButtonConfig } from "@/types/protocol";
import { AXIS_INDEX, isAxisHealthy } from "@/types/protocol";
import { cn } from "@/lib/utils";
import { AXIS_COLORS, getThemeColor } from "@/theme/colors";
import { useTheme } from "@/theme/ThemeProvider";

interface Props {
  snapshot: DeviceSnapshot;
  deviceConfig: UseDeviceConfigReturn;
}

function clampAxis(value: number): number {
  return Math.max(-32767, Math.min(32767, value));
}

function clampInt(value: number, min: number, max: number): number {
  if (!Number.isFinite(value)) return min;
  return Math.max(min, Math.min(max, Math.round(value)));
}

// ── HUD Crosshair Canvas ───────────────────────────────────────────────────────
function HudCanvas({ x, y, twist }: { x: number; y: number; twist: number }) {
  const ref = useRef<HTMLCanvasElement>(null);
  const { theme } = useTheme();

  useEffect(() => {
    const c = ref.current; if (!c) return;
    const ctx = c.getContext("2d")!;
    const W = c.width, H = c.height;
    const PAD = 8;
    const PW = W - PAD * 2, PH = H - PAD * 2;
    const colors = {
      surface: getThemeColor("surface"), grid: getThemeColor("grid"),
      center: getThemeColor("center"), marker: getThemeColor("marker"),
      accent: getThemeColor("accent"), accentGlow: getThemeColor("accentGlow"),
      twist: getThemeColor("axisTwist"),
    };

    ctx.clearRect(0, 0, W, H);

    // Background
    ctx.fillStyle = colors.surface; ctx.roundRect(0, 0, W, H, 8); ctx.fill();

    // Grid lines
    ctx.strokeStyle = colors.grid; ctx.lineWidth = 0.5;
    for (let i = 1; i < 4; i++) {
      ctx.beginPath(); ctx.moveTo(PAD + i * PW / 4, PAD); ctx.lineTo(PAD + i * PW / 4, PAD + PH); ctx.stroke();
      ctx.beginPath(); ctx.moveTo(PAD, PAD + i * PH / 4); ctx.lineTo(PAD + PW, PAD + i * PH / 4); ctx.stroke();
    }

    // Center axes
    ctx.strokeStyle = colors.center; ctx.lineWidth = 0.5;
    ctx.beginPath(); ctx.moveTo(W / 2, PAD); ctx.lineTo(W / 2, PAD + PH); ctx.stroke();
    ctx.beginPath(); ctx.moveTo(PAD, H / 2); ctx.lineTo(PAD + PW, H / 2); ctx.stroke();

    // Twist ring and pointer
    const twN = clampAxis(twist) / 32767;
    const cx = W / 2, cy = H / 2;
    const twistRadius = Math.min(PW, PH) / 2 - 10;
    const twistStart = -Math.PI / 2;
    const twistEnd = twistStart + twN * Math.PI;

    ctx.strokeStyle = colors.twist; ctx.globalAlpha = 0.12; ctx.lineWidth = 2;
    ctx.beginPath(); ctx.arc(cx, cy, twistRadius, 0, Math.PI * 2); ctx.stroke();

    ctx.globalAlpha = 0.65; ctx.strokeStyle = colors.twist; ctx.lineWidth = 3;
    ctx.beginPath();
    if (twN !== 0) ctx.arc(cx, cy, twistRadius, twistStart, twistEnd, twN < 0);
    ctx.stroke();

    const twMx = cx + Math.cos(twistEnd) * twistRadius;
    const twMy = cy + Math.sin(twistEnd) * twistRadius;
    ctx.globalAlpha = 0.35; ctx.strokeStyle = colors.twist; ctx.lineWidth = 1;
    ctx.beginPath(); ctx.moveTo(cx, cy); ctx.lineTo(twMx, twMy); ctx.stroke();
    ctx.globalAlpha = 1; ctx.fillStyle = colors.twist; ctx.beginPath(); ctx.arc(twMx, twMy, 4, 0, Math.PI * 2); ctx.fill();
    ctx.strokeStyle = colors.marker; ctx.lineWidth = 1.5; ctx.stroke();

    // Dot position
    const px = W / 2 + (clampAxis(x) / 32767) * (PW / 2 - 4);
    const py = H / 2 - (clampAxis(y) / 32767) * (PH / 2 - 4);

    // Glow
    const g = ctx.createRadialGradient(px, py, 0, px, py, 16);
    g.addColorStop(0, colors.accentGlow); g.addColorStop(1, "transparent");
    ctx.fillStyle = g; ctx.beginPath(); ctx.arc(px, py, 16, 0, Math.PI * 2); ctx.fill();

    // Dashed crosslines
    ctx.strokeStyle = colors.accent; ctx.globalAlpha = 0.4; ctx.lineWidth = 0.5; ctx.setLineDash([3, 3]);
    ctx.beginPath(); ctx.moveTo(px, H / 2); ctx.lineTo(px, py + 5); ctx.stroke();
    ctx.beginPath(); ctx.moveTo(W / 2, py); ctx.lineTo(px - 5, py); ctx.stroke();
    ctx.setLineDash([]);

    // Dot
    ctx.globalAlpha = 1; ctx.fillStyle = colors.accent; ctx.beginPath(); ctx.arc(px, py, 4, 0, Math.PI * 2); ctx.fill();
    ctx.globalAlpha = 0.3; ctx.strokeStyle = colors.accent; ctx.lineWidth = 2; ctx.stroke();
    ctx.globalAlpha = 1;
  }, [x, y, twist, theme]);

  return (
    <canvas
      ref={ref}
      width={150}
      height={150}
      className="rounded-lg flex-shrink-0"
    />
  );
}

// ── Axis bar (bidirectional) ───────────────────────────────────────────────────
function AxisBar({ label, value, color, healthy }: { label: string; value: number; color: string; healthy: boolean }) {
  const clamped = clampAxis(value);
  const pct = Math.abs(clamped) / 32767 * 50;
  const isPos = clamped >= 0;

  return (
    <div className="space-y-1">
      <div className="flex justify-between items-center">
        <span className="font-mono text-[11px] font-semibold" style={{ color }}>
          {label}
        </span>
        <div className="flex items-center gap-2">
          {!healthy && <span className="text-[9px] text-danger font-mono">⚠ FAULT</span>}
          <span className="font-mono text-[11px] text-content-primary tabular-nums w-14 text-right">
            {clamped.toLocaleString()}
          </span>
        </div>
      </div>
      <div className="relative h-1.5 bg-hud-border2 rounded-full overflow-hidden">
        <span className="absolute left-1/2 top-0 bottom-0 w-px bg-content-dim z-10" />
        <div
          className="absolute top-0 bottom-0 rounded-full transition-all duration-75"
          style={{
            backgroundColor: color,
            width: `${pct}%`,
            left: isPos ? "50%" : `${50 - pct}%`,
          }}
        />
      </div>
    </div>
  );
}

// ── Axis config controls ──────────────────────────────────────────────────────
function FieldRow({
  label, sub, children,
}: { label: string; sub?: string; children: React.ReactNode }) {
  return (
    <div className="flex min-h-10 items-center justify-between border-b border-hud-border py-1.5 last:border-0">
      <div>
        {sub ? (
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="cursor-help text-[11px] text-content-primary decoration-content-muted decoration-dotted underline-offset-4 hover:underline">
                {label}
              </div>
            </TooltipTrigger>
            <TooltipContent side="top" className="max-w-60 bg-hud-surface2 border-hud-border2 text-content-primary text-xs">
              {sub}
            </TooltipContent>
          </Tooltip>
        ) : (
          <div className="text-[11px] text-content-primary">{label}</div>
        )}
      </div>
      <div className="flex min-h-7 items-center gap-2">{children}</div>
    </div>
  );
}

function SliderField({
  label, sub, min, max, value, onChange, format, disabled = false,
}: {
  label: string; sub?: string;
  min: number; max: number; value: number;
  onChange: (v: number) => void;
  format?: (v: number) => string;
  disabled?: boolean;
}) {
  const display = format ? format(value) : String(value);
  return (
    <FieldRow label={label} sub={sub}>
      <Slider
        min={min}
        max={max}
        step={1}
        value={[value]}
        onValueChange={([v]) => { if (!disabled) onChange(v); }}
        disabled={disabled}
        className="w-24"
      />
      <span className="w-11 text-right font-mono text-[10px] tabular-nums text-content-primary">
        {display}
      </span>
    </FieldRow>
  );
}

function AxisConfigTab({
  axisId, deviceConfig,
}: { axisId: AxisId; deviceConfig: UseDeviceConfigReturn }) {
  const idx = AXIS_INDEX[axisId];
  const ax = deviceConfig.config.axes[idx];
  const disabled = !ax.enabled;
  const upd = (partial: Parameters<typeof deviceConfig.updateAxis>[1]) =>
    deviceConfig.updateAxis(idx, partial);
  const updateAxisToButton = (partial: Partial<AxisToButtonConfig>) => {
    const next = { ...ax.axis_to_button, ...partial };
    upd({ axis_to_button: next });
  };

  return (
    <div className="space-y-2">
      <Alert
        aria-hidden={!disabled}
        className={cn(
          "border-transparent bg-transparent py-1.5",
          !disabled && "invisible",
        )}
      >
        <AlertDescription className="text-center text-xs text-warn">
          Eixo desabilitado. Reative o eixo para alterar parâmetros.
        </AlertDescription>
      </Alert>

      <div className="grid gap-x-4 md:grid-cols-[0.85fr_1.15fr]">
        <div>
          <FieldRow label="Habilitado">
            <Switch
              checked={ax.enabled}
              onCheckedChange={(v) => upd({ enabled: v })}
              className="data-[state=checked]:bg-cyan"
            />
          </FieldRow>
          <FieldRow label="Invertido" sub="Inverte a saída do eixo">
            <Switch
              checked={ax.inverted}
              onCheckedChange={(v) => upd({ inverted: v })}
              disabled={disabled}
              className="data-[state=checked]:bg-cyan disabled:opacity-40"
            />
          </FieldRow>
          <FieldRow label="Reset EMA na deadzone" sub="Recomendado para Twist">
            <Switch
              checked={ax.reset_ema_on_dz}
              onCheckedChange={(v) => upd({ reset_ema_on_dz: v })}
              disabled={disabled}
              className="data-[state=checked]:bg-cyan disabled:opacity-40"
            />
          </FieldRow>
        </div>

        <div className={cn(disabled && "opacity-55")}>
          <SliderField
            label="EMA α"
            sub="permille (1000 = sem filtro)"
            min={1}
            max={1000}
            value={ax.ema_permille}
            onChange={(v) => upd({ ema_permille: v })}
            format={(v) => `${(v / 1000).toFixed(3)}`}
            disabled={disabled}
          />
          <SliderField
            label="Deadzone"
            sub="permille (0 = sem deadzone)"
            min={0}
            max={200}
            value={ax.deadzone_permille}
            onChange={(v) => upd({ deadzone_permille: v })}
            format={(v) => `${(v / 10).toFixed(1)}%`}
            disabled={disabled}
          />
          <SliderField
            label="Center offset"
            sub="Ajuste fino do zero após calibração"
            min={-200}
            max={200}
            value={ax.center_offset_permille}
            onChange={(v) => upd({ center_offset_permille: v })}
            format={(v) => `${(v / 10).toFixed(1)}%`}
            disabled={disabled}
          />
          <SliderField
            label="Travel limit"
            sub="Percentual simétrico a partir do centro"
            min={1}
            max={100}
            value={ax.travel.travel_limit_pct}
            onChange={(v) => upd({ travel: { ...ax.travel, travel_limit_pct: v } })}
            format={(v) => `${v}%`}
            disabled={disabled}
          />
        </div>
      </div>

      <div className={cn(
        "space-y-2 border-t border-hud-border pt-2",
        disabled && "opacity-55",
      )}>
        <div className="flex items-end justify-between gap-3">
          <div>
            <Tooltip>
              <TooltipTrigger asChild>
                <span className="cursor-help text-[11px] text-content-primary decoration-content-muted decoration-dotted underline-offset-4 hover:underline">
                  Botão virtual
                </span>
              </TooltipTrigger>
              <TooltipContent side="top" className="max-w-60 bg-hud-surface2 border-hud-border2 text-content-primary text-xs">
                Ativa um botão HID quando o eixo cruza o limiar.
              </TooltipContent>
            </Tooltip>
          </div>
          <Switch
            checked={ax.axis_to_button.enabled}
            onCheckedChange={(v) => updateAxisToButton({ enabled: v })}
            disabled={disabled}
            className="data-[state=checked]:bg-cyan disabled:opacity-40"
          />
        </div>

        <div className="grid grid-cols-2 items-start gap-3">
          <div>
            <div className="mb-1 h-3 text-left text-[10px] leading-3 text-content-muted">Limiar botão</div>
            <div className="grid h-7 grid-cols-[1fr_auto] items-center gap-2">
              <Slider
                min={0}
                max={1000}
                step={1}
                value={[ax.axis_to_button.threshold_permille]}
                onValueChange={([v]) => updateAxisToButton({ threshold_permille: v })}
                disabled={disabled || !ax.axis_to_button.enabled}
                className="flex-1"
              />
              <span className="w-9 text-right font-mono text-[10px] tabular-nums text-content-primary">
                {(ax.axis_to_button.threshold_permille / 10).toFixed(0)}%
              </span>
            </div>
          </div>

          <div className="grid grid-cols-[1fr_0.55fr] items-start gap-2">
            <div>
              <div className="mb-1 h-3 text-left text-[10px] leading-3 text-content-muted">Direção botão</div>
              <Select
                value={ax.axis_to_button.direction}
                onValueChange={(value) =>
                  updateAxisToButton({ direction: value as AxisToButtonConfig["direction"] })
                }
                disabled={disabled || !ax.axis_to_button.enabled}
              >
                <SelectTrigger className="h-7 w-full bg-hud-surface2 border-hud-border2 font-mono text-[11px] text-content-primary disabled:opacity-40">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent className="bg-hud-surface2 border-hud-border2 text-content-primary">
                  <SelectItem value="Positive" className="font-mono text-xs">Positivo</SelectItem>
                  <SelectItem value="Negative" className="font-mono text-xs">Negativo</SelectItem>
                  <SelectItem value="Both" className="font-mono text-xs">Ambos</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div>
              <div className="mb-1 h-3 text-left text-[10px] leading-3 text-content-muted">Botão</div>
              <Input
                type="number"
                min={0}
                max={31}
                value={ax.axis_to_button.button_index}
                onChange={(e) => {
                  if (disabled || !ax.axis_to_button.enabled) return;
                  updateAxisToButton({ button_index: clampInt(Number(e.target.value), 0, 31) });
                }}
                disabled={disabled || !ax.axis_to_button.enabled}
                className="h-7 w-full bg-hud-surface2 border-hud-border2 text-right font-mono text-[11px] text-content-primary disabled:opacity-40"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

// ── Main Axes page ─────────────────────────────────────────────────────────────
export function AxesPage({ snapshot, deviceConfig }: Props) {
  const { axes } = snapshot;
  const { dirty, loading, error, save, reload } = deviceConfig;
  const ax = axes ?? { x: 0, y: 0, twist: 0, unhealthy_mask: 0 };

  const hx = isAxisHealthy(ax.unhealthy_mask, "X");
  const hy = isAxisHealthy(ax.unhealthy_mask, "Y");
  const htw = isAxisHealthy(ax.unhealthy_mask, "Twist");

  return (
    <div className="p-3 h-full">
      <div className="mx-auto flex h-full max-w-6xl flex-col gap-2 min-w-0">

        <Card className="bg-hud-surface border-hud-border2">
          <CardContent className="space-y-2.5 p-3">
            {error && (
              <Alert className="bg-danger/10 border-danger/40 py-1.5">
                <AlertDescription className="text-xs text-danger">{error}</AlertDescription>
              </Alert>
            )}

            <Tabs defaultValue="X">
              <div className="grid grid-cols-[1fr_150px] items-center gap-4">
                <div className="space-y-3">
                  <AxisBar label="X"     value={ax.x}     color={AXIS_COLORS.X}     healthy={hx} />
                  <AxisBar label="Y"     value={ax.y}     color={AXIS_COLORS.Y}     healthy={hy} />
                  <AxisBar label="TWIST" value={ax.twist} color={AXIS_COLORS.Twist} healthy={htw} />
                  <TabsList className="bg-hud-surface2 border border-hud-border2 h-8">
                    {(["X", "Y", "Twist"] as AxisId[]).map((axisId) => (
                      <TabsTrigger
                        key={axisId}
                        value={axisId}
                        className={cn(
                          "h-6 w-14 px-0 text-xs font-mono font-semibold data-[state=active]:text-content-inverse",
                          axisId === "X" && "data-[state=active]:bg-axis-x",
                          axisId === "Y" && "data-[state=active]:bg-axis-y",
                          axisId === "Twist" && "data-[state=active]:bg-axis-tw",
                        )}
                      >
                        {axisId}
                      </TabsTrigger>
                    ))}
                  </TabsList>
                </div>
                <HudCanvas x={ax.x} y={ax.y} twist={ax.twist} />
              </div>

              {(["X", "Y", "Twist"] as AxisId[]).map((axisId) => (
                <TabsContent key={axisId} value={axisId} className="mt-0">
                  <AxisConfigTab axisId={axisId} deviceConfig={deviceConfig} />
                </TabsContent>
              ))}
            </Tabs>

            <Alert className={cn(
              "py-1.5",
              dirty
                ? "bg-warn/10 border-warn/40 animate-fade-in"
                : "bg-hud-surface2 border-hud-border2"
            )}>
              <AlertDescription className="flex items-center justify-between gap-3">
                <span className={cn("text-xs", dirty ? "text-warn" : "text-content-muted")}>
                  {dirty ? "Alterações não salvas no flash" : "Sem alterações pendentes"}
                </span>
                <div className="flex gap-2">
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={reload}
                    disabled={!dirty || loading}
                    className="h-7 text-xs text-content-muted hover:text-content-primary disabled:opacity-40"
                  >
                    Descartar
                  </Button>
                  <Button
                    size="sm"
                    onClick={save}
                    disabled={!dirty || loading}
                    className="h-7 text-xs bg-ok/10 border border-ok/30 text-ok hover:bg-ok/20 disabled:opacity-40"
                  >
                    Salvar
                  </Button>
                </div>
              </AlertDescription>
            </Alert>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
