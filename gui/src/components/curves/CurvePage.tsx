import { useRef, useState } from "react";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Slider } from "@/components/ui/slider";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import type { UseDeviceConfigReturn } from "@/hooks/useDeviceConfig";
import type { AxisId, ResponseCurveData } from "@/types/protocol";
import { AXIS_INDEX } from "@/types/protocol";
import { CurveEditor } from "./CurveEditor";
import { cn } from "@/lib/utils";

interface Props {
  deviceConfig: UseDeviceConfigReturn;
}

const CURVE_SETUPS = [
  {
    id: "linear",
    label: "Linear",
    response_curve: {
      point_left: { x: -500, y: -500 },
      point_right: { x: 500, y: 500 },
    },
  },
  {
    id: "smooth",
    label: "Suave",
    response_curve: {
      point_left: { x: -400, y: -250 },
      point_right: { x: 400, y: 250 },
    },
  },
  {
    id: "center",
    label: "Centro",
    response_curve: {
      point_left: { x: -300, y: -150 },
      point_right: { x: 300, y: 150 },
    },
  },
  {
    id: "s",
    label: "S-curve",
    response_curve: {
      point_left: { x: -250, y: -600 },
      point_right: { x: 250, y: 600 },
    },
  },
] as const;

interface CurveHistoryItem {
  axisIndex: 0 | 1 | 2;
  response_curve: ResponseCurveData;
  deadzone_permille: number;
}

function curvesEqual(a: ResponseCurveData, b: ResponseCurveData): boolean {
  return (
    a.point_left.x === b.point_left.x &&
    a.point_left.y === b.point_left.y &&
    a.point_right.x === b.point_right.x &&
    a.point_right.y === b.point_right.y
  );
}

function AxisCurveView({ axisId, deviceConfig }: { axisId: AxisId; deviceConfig: UseDeviceConfigReturn }) {
  const idx = AXIS_INDEX[axisId];
  const ax = deviceConfig.config.axes[idx];
  const disabled = !ax.enabled;

  return (
    <div className="space-y-3">
      {disabled && (
        <Alert className="bg-warn/10 border-warn/40 py-2">
          <AlertDescription className="text-xs text-warn">
            Eixo desabilitado. Reative em Eixos para editar a curva.
          </AlertDescription>
        </Alert>
      )}

      <CurveEditor
        axisIndex={idx}
        responseCurve={ax.response_curve}
        deadzonePermille={ax.deadzone_permille}
        disabled={disabled}
      />
    </div>
  );
}

export function CurvePage({ deviceConfig }: Props) {
  const [axis, setAxis] = useState<AxisId>("X");
  const [curveHistory, setCurveHistory] = useState<CurveHistoryItem[]>([]);
  const sliderStartRef = useRef<CurveHistoryItem | null>(null);
  const { dirty, loading, error, save, reload } = deviceConfig;
  const activeIdx = AXIS_INDEX[axis];
  const activeAxis = deviceConfig.config.axes[activeIdx];
  const activeDisabled = !activeAxis.enabled;
  const activeSetup = CURVE_SETUPS.find((setup) =>
    curvesEqual(activeAxis.response_curve, setup.response_curve),
  );
  const customSetupActive = !activeSetup;

  const pushHistory = (item: CurveHistoryItem) => {
    setCurveHistory((history) => [...history.slice(-29), item]);
  };

  const snapshotActiveCurve = (): CurveHistoryItem => ({
    axisIndex: activeIdx,
    response_curve: activeAxis.response_curve,
    deadzone_permille: activeAxis.deadzone_permille,
  });

  const undoCurveChange = () => {
    const last = curveHistory[curveHistory.length - 1];
    if (!last) return;

    deviceConfig.updateAxis(last.axisIndex, {
      response_curve: last.response_curve,
      deadzone_permille: last.deadzone_permille,
    });
    setCurveHistory((history) => history.slice(0, -1));
  };

  return (
    <div className="mx-auto max-w-3xl space-y-3 p-4">
      {error && (
        <Alert className="bg-danger/10 border-danger/40 py-2">
          <AlertDescription className="text-xs text-danger">{error}</AlertDescription>
        </Alert>
      )}

      <Tabs value={axis} onValueChange={(value) => setAxis(value as AxisId)}>
        <Card className="bg-hud-surface border-hud-border2">
          <CardContent className="space-y-3 px-4 py-4">
            <div className="flex items-end justify-between gap-3">
              <div>
                <div className="mb-1 text-[10px] uppercase tracking-widest text-content-muted">Eixo</div>
                <TabsList className="h-8 border border-hud-border2 bg-hud-surface2">
                  {(["X", "Y", "Twist"] as AxisId[]).map((axisId) => (
                    <TabsTrigger
                      key={axisId}
                      value={axisId}
                      className="h-6 w-14 px-0 text-xs font-mono font-semibold data-[state=active]:text-content-inverse"
                    >
                      {axisId}
                    </TabsTrigger>
                  ))}
                </TabsList>
              </div>

              <div className="ml-auto">
                <div className="mb-1 text-[10px] uppercase tracking-widest text-content-muted">Setup</div>
                <div className="flex items-center gap-1.5">
                  {CURVE_SETUPS.map((setup) => {
                    const active = activeSetup?.id === setup.id;

                    return (
                      <Button
                        key={setup.id}
                        size="sm"
                        variant="outline"
                        disabled={activeDisabled}
                        onClick={() => {
                          if (!activeDisabled && !active) {
                            pushHistory(snapshotActiveCurve());
                            deviceConfig.updateAxis(activeIdx, {
                              response_curve: { ...setup.response_curve },
                            });
                          }
                        }}
                        className={cn(
                          "h-8 px-2.5 text-[10px] font-mono",
                          active
                            ? "border-cyan/50 bg-cyan-dim text-cyan"
                            : "border-hud-border2 bg-transparent text-content-muted hover:text-content-primary",
                        )}
                      >
                        {setup.label}
                      </Button>
                    );
                  })}
                  <Button
                    size="sm"
                    variant="outline"
                    disabled={activeDisabled}
                    className={cn(
                      "h-8 px-2.5 text-[10px] font-mono",
                      customSetupActive
                        ? "border-cyan/50 bg-cyan-dim text-cyan"
                        : "border-hud-border2 bg-transparent text-content-muted hover:text-content-primary",
                    )}
                  >
                    Personalizado
                  </Button>
                </div>
              </div>
            </div>

            <div className={cn(activeDisabled && "opacity-55")}>
              <div className="mb-1 text-[10px] uppercase tracking-widest text-content-muted">Deadzone</div>
              <div className="flex items-center gap-3">
                <Slider
                  min={0}
                  max={200}
                  step={1}
                  value={[activeAxis.deadzone_permille]}
                  disabled={activeDisabled}
                  onValueChange={([value]) => {
                    if (activeDisabled) return;
                    if (!sliderStartRef.current) sliderStartRef.current = snapshotActiveCurve();
                    deviceConfig.updateAxis(activeIdx, { deadzone_permille: value });
                  }}
                  onValueCommit={([value]) => {
                    const start = sliderStartRef.current;
                    if (start && start.deadzone_permille !== value) pushHistory(start);
                    sliderStartRef.current = null;
                  }}
                  className="flex-1"
                />
                <span className="w-12 text-right font-mono text-[10px] text-content-muted">
                  {(activeAxis.deadzone_permille / 10).toFixed(1)}%
                </span>
              </div>
            </div>

            <div className="flex justify-end">
              <Button
                size="sm"
                variant="ghost"
                onClick={undoCurveChange}
                disabled={!curveHistory.length}
                className="h-7 px-3 text-[11px] text-content-dim hover:text-content-primary"
              >
                Desfazer
              </Button>
            </div>

            {(["X", "Y", "Twist"] as AxisId[]).map((axisId) => (
              <TabsContent key={axisId} value={axisId} className="mt-0">
                <AxisCurveView axisId={axisId} deviceConfig={deviceConfig} />
              </TabsContent>
            ))}

            <Alert
              className={cn(
                "py-2",
                dirty
                  ? "bg-warn/10 border-warn/40 animate-fade-in"
                  : "bg-hud-surface2 border-hud-border2",
              )}
            >
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
      </Tabs>
    </div>
  );
}
