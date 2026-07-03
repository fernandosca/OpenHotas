import { useState } from "react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsList } from "@/components/ui/tabs";
import { AXIS_IDS, AxisTabTrigger } from "@/components/axes/AxisTabTrigger";
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter,
} from "@/components/ui/dialog";
import { Alert, AlertDescription } from "@/components/ui/alert";
import type { DeviceSnapshot } from "@/hooks/useDevicePolling";
import type { UseDeviceConfigReturn } from "@/hooks/useDeviceConfig";
import { useCalibration } from "@/hooks/useCalibration";
import type { AxisId } from "@/types/protocol";
import { AXIS_INDEX } from "@/types/protocol";
import { cn } from "@/lib/utils";
import { AXIS_COLORS } from "@/theme/colors";

interface Props {
  snapshot: DeviceSnapshot;
  deviceConfig: UseDeviceConfigReturn;
}

const STEP_LABELS = {
  idle:             "Aguardando",
  session_open:     "Sessão aberta",
  min_captured:     "Mín capturado",
  center_captured:  "Centro capturado",
  complete:         "Completo",
};

const STEP_INSTR = {
  idle:            "Selecione um eixo e pressione Iniciar para começar a sessão de calibração.",
  session_open:    "Mova o eixo até a posição MÍNIMA física (endstop) e pressione Capturar Min.",
  min_captured:    "Retorne o eixo ao CENTRO (posição de descanso) e pressione Capturar Centro.",
  center_captured: "Mova o eixo até a posição MÁXIMA física (endstop) e pressione Capturar Max.",
  complete:        "Calibração aplicada ao runtime. Pressione Salvar no flash para persistir.",
};

const STEP_ORDER = ["idle", "session_open", "min_captured", "center_captured", "complete"];

export function Calibration({ snapshot, deviceConfig }: Props) {
  const [axis, setAxis] = useState<AxisId>("X");
  const [confirmReset, setConfirmReset] = useState(false);
  const cal = useCalibration();
  const color = AXIS_COLORS[axis];
  const axisEnabled = deviceConfig.config.axes[AXIS_INDEX[axis]].enabled;

  // Current raw value from snapshot
  const rawMap: Record<AxisId, number> = {
    X:     (snapshot.axes?.x     ?? 0) + 16384, // shift i16 → display u16
    Y:     (snapshot.axes?.y     ?? 0) + 16384,
    Twist: (snapshot.axes?.twist ?? 0) + 16384,
  };
  const raw = rawMap[axis];

  const stepIdx = STEP_ORDER.indexOf(cal.step);

  async function handleAction() {
    if (!axisEnabled) return;
    if (cal.step === "idle")            return cal.start(axis);
    if (cal.step === "session_open")    return cal.captureMin(axis, raw);
    if (cal.step === "min_captured")    return cal.captureCenter(axis, raw);
    if (cal.step === "center_captured") return cal.captureMax(axis, raw);
  }

  const actionLabel = {
    idle:            "Iniciar",
    session_open:    "Capturar Min",
    min_captured:    "Capturar Centro",
    center_captured: "Capturar Max",
    complete:        "Completo",
  }[cal.step];
  const actionDisabled = cal.busy || !snapshot.connected || !axisEnabled || cal.step === "complete";
  const statusLabel = cal.persisted ? "Salvo no flash" : STEP_LABELS[cal.step];
  const instruction = cal.persisted
    ? "Calibração salva no flash. Esta configuração será mantida após reiniciar."
    : STEP_INSTR[cal.step];

  return (
    <div className="p-4 h-full">
      <Card className="mx-auto w-full max-w-6xl bg-hud-surface border-hud-border2">
        <CardContent className="p-4 space-y-4">
          <div className="grid grid-cols-2 items-end gap-3">
            <div>
              <div className="mb-1 text-[10px] uppercase tracking-widest text-content-muted">Eixo</div>
              <Tabs value={axis} onValueChange={(v) => { setAxis(v as AxisId); cal.reset(); }}>
                <TabsList className="bg-hud-surface2 border border-hud-border2 h-8">
                  {AXIS_IDS.map((ax) => (
                    <AxisTabTrigger
                      key={ax}
                      axis={ax}
                      variant="subtle"
                      enabled={deviceConfig.config.axes[AXIS_INDEX[ax]].enabled}
                    />
                  ))}
                </TabsList>
              </Tabs>
            </div>

            <div>
              <div className="mb-1 text-[10px] uppercase tracking-widest text-content-muted">Status</div>
              <div className="flex">
                <Badge
                  variant="outline"
                  className={cn(
                    "h-8 w-full justify-center text-[10px] font-mono",
                    cal.step === "complete" ? "border-ok/40 text-ok bg-ok/10" :
                    cal.step === "idle"     ? "border-content-dim text-content-muted" :
                                              "border-warn/40 text-warn bg-warn/10"
                  )}
                >
                  {statusLabel}
                </Badge>
              </div>
            </div>
          </div>

          <div className="flex items-center gap-0">
            {STEP_ORDER.map((s, i) => {
              const done   = i < stepIdx;
              const active = i === stepIdx;
              return (
                <div key={s} className="flex items-center flex-1 last:flex-none">
                  <div className={cn(
                    "w-6 h-6 rounded-full flex items-center justify-center text-[10px] font-mono border flex-shrink-0",
                    done   && "border-ok bg-ok/10 text-ok",
                    active && "border-cyan text-cyan bg-cyan-dim",
                    !done && !active && "border-hud-border2 text-content-dim",
                  )}>
                    {done ? "✓" : i + 1}
                  </div>
                  {i < STEP_ORDER.length - 1 && (
                    <div className={cn(
                      "flex-1 h-px mx-1",
                      done ? "bg-ok/40" : "bg-hud-border2"
                    )} />
                  )}
                </div>
              );
            })}
          </div>

          <Alert className="bg-hud-surface2 border-hud-border2 py-2">
            <AlertDescription className="text-center text-xs text-content-primary">
              {axisEnabled
                ? instruction
                : "Eixo desabilitado. Reative em Eixos para calibrar."}
            </AlertDescription>
          </Alert>

          <div className="grid grid-cols-3 gap-2">
            {(["min", "center", "max"] as const).map((pt, i) => {
              const val = cal.captures[pt];
              const isTarget =
                (pt === "min"    && cal.step === "session_open") ||
                (pt === "center" && cal.step === "min_captured") ||
                (pt === "max"    && cal.step === "center_captured");
              return (
                <div key={pt} className={cn(
                  "rounded-lg border p-3 text-center transition-all",
                  isTarget && "border-cyan/50 bg-cyan-dim",
                  !isTarget && val !== null && "border-ok/30 bg-ok/5",
                  !isTarget && val === null && "border-hud-border2 bg-hud-surface2",
                )}>
                  <div className={cn(
                    "text-[10px] uppercase tracking-widest mb-1",
                    isTarget ? "text-cyan" : "text-content-muted"
                  )}>
                    {["Mín", "Centro", "Máx"][i]}
                  </div>
                  <div className={cn(
                    "font-mono text-base font-semibold",
                    val !== null ? "text-ok" : "text-content-dim"
                  )}>
                    {val !== null ? val : "—"}
                  </div>
                </div>
              );
            })}
          </div>

          <div className="grid grid-cols-[1fr_auto_auto] items-center gap-2">
            {cal.step !== "idle" && (
              <Button
                variant="ghost"
                onClick={() => setConfirmReset(true)}
                className="justify-self-start text-xs h-8 text-content-muted hover:text-danger hover:bg-danger/10"
              >
                Resetar
              </Button>
            )}
            {cal.step === "idle" && <div />}

            <div className="justify-self-end">
              {cal.step === "complete" && !cal.persisted && (
                <Button
                  variant="outline"
                  onClick={cal.persist}
                  disabled={cal.busy || !axisEnabled}
                  className="text-xs h-8 border-ok/30 text-ok bg-ok/5 hover:bg-ok/10"
                >
                  Salvar no flash
                </Button>
              )}
            </div>

            <Button
              onClick={handleAction}
              disabled={actionDisabled}
              className="h-8 w-32 bg-cyan-dim border border-cyan/30 text-cyan hover:bg-cyan/20 text-xs font-mono disabled:opacity-45"
              style={{ color }}
            >
              {cal.busy ? "Aguarde…" : actionLabel}
            </Button>
          </div>

          {cal.error && (
            <p className="text-[10px] font-mono text-danger">{cal.error}</p>
          )}

          <div className="border-t border-hud-border pt-3">
            <div className="mb-3 text-center text-[10px] uppercase tracking-widest text-content-muted">Notas</div>
            <ul className="grid grid-cols-3 gap-3 text-center text-[clamp(10px,1vw,12px)] text-content-muted">
              <li className="flex min-w-0 justify-center gap-2 whitespace-nowrap"><span className="text-cyan">→</span> <span className="min-w-0">FinishCalibration aplica ao runtime</span></li>
              <li className="flex min-w-0 justify-center gap-2 whitespace-nowrap"><span className="text-warn">!</span> <span className="min-w-0">Não grava no flash automaticamente</span></li>
              <li className="flex min-w-0 justify-center gap-2 whitespace-nowrap"><span className="text-ok">✓</span> <span className="min-w-0">Salvar no flash para persistir</span></li>
            </ul>
          </div>
        </CardContent>
      </Card>

      {/* Reset confirm dialog */}
      <Dialog open={confirmReset} onOpenChange={setConfirmReset}>
        <DialogContent className="bg-hud-surface border-hud-border2 text-content-primary">
          <DialogHeader>
            <DialogTitle>Resetar calibração?</DialogTitle>
            <DialogDescription className="text-content-muted">
              O progresso atual será descartado. A calibração no firmware não será afetada.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="ghost" onClick={() => setConfirmReset(false)} className="text-content-muted">
              Cancelar
            </Button>
            <Button
              onClick={() => { cal.reset(); setConfirmReset(false); }}
              className="bg-danger/20 border border-danger/40 text-danger hover:bg-danger/30"
            >
              Resetar
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
