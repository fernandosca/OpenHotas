import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { UnsavedChangesBar } from "@/components/config/UnsavedChangesBar";
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from "@/components/ui/select";
import type { DeviceSnapshot } from "@/hooks/useDevicePolling";
import type { UseDeviceConfigReturn } from "@/hooks/useDeviceConfig";
import { cn } from "@/lib/utils";

interface Props {
  snapshot: DeviceSnapshot;
  deviceConfig: UseDeviceConfigReturn;
}

function isPressed(mask: number, index: number): boolean {
  return Math.floor(mask / 2 ** index) % 2 === 1;
}

function ButtonCell({ index, pressed }: { index: number; pressed: boolean }) {
  return (
    <div
      className={cn(
        "h-9 w-9 rounded-md border flex flex-col items-center justify-center gap-0.5",
        "transition-colors duration-75 font-mono",
        pressed
          ? "bg-cyan-dim border-cyan/50 text-cyan shadow-[0_0_10px_var(--cyan-glow)]"
          : "bg-hud-surface2 border-hud-border2 text-content-dim"
      )}
    >
      <span className="text-[11px] font-semibold">{index}</span>
      <span className={cn("h-1 w-1 rounded-full", pressed ? "bg-cyan" : "bg-content-dim")} />
    </div>
  );
}

function FieldRow({
  label, sub, children,
}: { label: string; sub?: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between border-b border-hud-border py-2.5 last:border-0">
      <div>
        <div className="text-xs text-content-primary">{label}</div>
        {sub && <div className="mt-0.5 text-[10px] text-content-muted">{sub}</div>}
      </div>
      <div className="flex items-center gap-3">{children}</div>
    </div>
  );
}

export function ButtonsPage({ snapshot, deviceConfig }: Props) {
  const mask = snapshot.buttons?.mask ?? 0;
  const buttonConfig = deviceConfig.config.buttons;
  const { dirty, loading, error, save, reload } = deviceConfig;

  return (
    <div className="h-full p-4">
      <div className="mx-auto flex h-full w-full max-w-6xl flex-col">
        <Card className="bg-hud-surface border-hud-border2">
          <CardHeader className="px-4 pt-3 pb-2">
            <CardTitle className="text-[11px] uppercase tracking-widest text-content-muted">
              Botões · 32 canais
            </CardTitle>
          </CardHeader>
          <CardContent className="px-4 pb-4">
            <div className="flex justify-center">
              <div className="grid w-fit grid-cols-8 gap-1.5 sm:grid-cols-16">
                {Array.from({ length: 32 }, (_, index) => (
                  <ButtonCell key={index} index={index} pressed={isPressed(mask, index)} />
                ))}
              </div>
            </div>
          </CardContent>

          <CardContent className="border-t border-hud-border px-4 pb-4 pt-3">
            <div className="mb-2 text-[10px] uppercase tracking-widest text-content-muted">Configuração do botão</div>
            <FieldRow label="Debounce">
              <Select
                value={String(buttonConfig.debounce_ms)}
                onValueChange={(value) =>
                  deviceConfig.updateButtons({ debounce_ms: Number(value) })
                }
              >
                <SelectTrigger className="h-7 w-28 bg-hud-surface2 border-hud-border2 font-mono text-xs text-content-primary">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent className="bg-hud-surface2 border-hud-border2 text-content-primary">
                  {[1, 2, 5, 10, 20].map((value) => (
                    <SelectItem key={value} value={String(value)} className="font-mono text-xs">
                      {value} ms
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </FieldRow>

            <UnsavedChangesBar dirty={dirty} loading={loading} error={error}
              onDiscard={reload} onSave={save} className="mt-3" />
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
