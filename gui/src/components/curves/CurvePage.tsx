import { useRef, useState } from "react";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Slider } from "@/components/ui/slider";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Tabs, TabsContent, TabsList } from "@/components/ui/tabs";
import { AXIS_IDS, AxisTabTrigger } from "@/components/axes/AxisTabTrigger";
import { UnsavedChangesBar } from "@/components/config/UnsavedChangesBar";
import type { UseDeviceConfigReturn } from "@/hooks/useDeviceConfig";
import type { DeviceSnapshot } from "@/hooks/useDevicePolling";
import type { AxisId, ResponseCurveData } from "@/types/protocol";
import { AXIS_INDEX } from "@/types/protocol";
import { CurveEditor } from "./CurveEditor";
import { cn } from "@/lib/utils";

interface Props {
  deviceConfig: UseDeviceConfigReturn;
  snapshot?: DeviceSnapshot;
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

function AxisCurveView({
  axisId,
  deviceConfig,
  currentOutput,
}: {
  axisId: AxisId;
  deviceConfig: UseDeviceConfigReturn;
  currentOutput: number | null;
}) {
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
        currentOutput={ax.inverted && currentOutput !== null ? -currentOutput : currentOutput}
        disabled={disabled}
      />
    </div>
  );
}

export function CurvePage({ deviceConfig, snapshot }: Props) {
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
  const currentOutputs: Record<AxisId, number | null> = {
    X: snapshot?.axes?.x ?? null,
    Y: snapshot?.axes?.y ?? null,
    Twist: snapshot?.axes?.twist ?? null,
  };

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
    <div className="mx-auto h-full w-full max-w-6xl space-y-3 p-4">
      <Tabs value={axis} onValueChange={(value) => setAxis(value as AxisId)}>
        <Card className="bg-hud-surface border-hud-border2">
          <CardContent className="space-y-3 px-4 py-4">
            <div className="grid w-full grid-cols-2 items-start gap-3">
              <div>
                <div className="mb-1 text-[10px] uppercase tracking-widest text-content-muted">Eixo</div>
                <TabsList className="h-8 border border-hud-border2 bg-hud-surface2">
                  {AXIS_IDS.map((axisId) => (
                    <AxisTabTrigger
                      key={axisId}
                      axis={axisId}
                    />
                  ))}
                </TabsList>
              </div>

              <div>
                <div className="mb-1 text-[10px] uppercase tracking-widest text-content-muted">Setup</div>
                <Select
                  value={activeSetup?.id ?? "custom"}
                  disabled={activeDisabled}
                  onValueChange={(value) => {
                    const setup = CURVE_SETUPS.find((item) => item.id === value);
                    if (!setup || setup.id === activeSetup?.id) return;
                    pushHistory(snapshotActiveCurve());
                    deviceConfig.updateAxis(activeIdx, {
                      response_curve: { ...setup.response_curve },
                    });
                  }}
                >
                  <SelectTrigger className="h-8 w-full border-hud-border2 bg-hud-surface2 font-mono text-[10px] text-cyan">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent className="border-hud-border2 bg-hud-raised text-content-primary">
                    {CURVE_SETUPS.map((setup) => (
                      <SelectItem key={setup.id} value={setup.id} className="font-mono text-xs">
                        {setup.label}
                      </SelectItem>
                    ))}
                    <SelectItem value="custom" disabled className="font-mono text-xs">
                      Personalizado
                    </SelectItem>
                  </SelectContent>
                </Select>
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

            {AXIS_IDS.map((axisId) => (
              <TabsContent key={axisId} value={axisId} className="mt-0">
                <AxisCurveView
                  axisId={axisId}
                  deviceConfig={deviceConfig}
                  currentOutput={currentOutputs[axisId]}
                />
              </TabsContent>
            ))}

            <UnsavedChangesBar dirty={dirty} loading={loading} error={error}
              onDiscard={reload} onSave={save} />
          </CardContent>
        </Card>
      </Tabs>
    </div>
  );
}
