import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import {
  Dialog, DialogContent, DialogHeader, DialogTitle,
  DialogDescription, DialogFooter,
} from "@/components/ui/dialog";
import type { UseDeviceConfigReturn } from "@/hooks/useDeviceConfig";
import { factoryReset, reboot } from "@/lib/tauri";
import { cn } from "@/lib/utils";

interface Props {
  deviceConfig: UseDeviceConfigReturn;
}

type DangerAction = "reboot" | "defaults" | "factory";

export function SettingsPage({ deviceConfig }: Props) {
  const [dangerDialog, setDangerDialog] = useState<DangerAction | null>(null);
  const [actionBusy, setActionBusy] = useState(false);

  async function handleDanger() {
    if (!dangerDialog) return;
    setActionBusy(true);
    try {
      if (dangerDialog === "reboot") await reboot();
      if (dangerDialog === "defaults") await deviceConfig.reloadDefaults();
      if (dangerDialog === "factory") await factoryReset();
    } finally {
      setActionBusy(false);
      setDangerDialog(null);
    }
  }

  return (
    <div className="flex h-full flex-col p-4">
      <Card className="flex min-h-0 flex-1 flex-col bg-hud-surface border-hud-border2">
        <CardContent className="flex min-h-0 flex-1 flex-col px-4 py-4">
          <div className="mb-3 text-[11px] uppercase tracking-widest text-slate-500">
            Configurações
          </div>

          <div className="mt-auto border-t border-hud-border pt-3">
            <div className="mb-2 text-[10px] uppercase tracking-widest text-danger/70">
              Ações do dispositivo
            </div>
            <div className="flex flex-col gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setDangerDialog("reboot")}
                className="h-8 text-xs border-hud-border2 text-slate-400 hover:border-warn/40 hover:text-warn"
              >
                ↺ Reboot
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setDangerDialog("defaults")}
                className="h-8 text-xs border-hud-border2 text-slate-400 hover:border-warn/40 hover:text-warn"
              >
                Carregar defaults
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setDangerDialog("factory")}
                className="h-8 text-xs border-danger/30 text-danger/70 hover:bg-danger/10 hover:text-danger"
              >
                ⚠ Factory reset
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      <Dialog open={dangerDialog !== null} onOpenChange={(open) => !open && setDangerDialog(null)}>
        <DialogContent className="bg-hud-surface border-hud-border2 text-slate-200">
          <DialogHeader>
            <DialogTitle>
              {dangerDialog === "reboot" && "Reboot do dispositivo?"}
              {dangerDialog === "defaults" && "Carregar defaults?"}
              {dangerDialog === "factory" && "Factory reset?"}
            </DialogTitle>
            <DialogDescription className="text-slate-400 text-xs">
              {dangerDialog === "factory"
                ? "Apaga a configuração e calibração do flash. Não pode ser desfeito."
                : dangerDialog === "reboot"
                ? "O firmware será reiniciado. A conexão serial será reestabelecida automaticamente."
                : "Os defaults serão carregados no runtime (não grava no flash)."}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="ghost"
              onClick={() => setDangerDialog(null)}
              className="text-slate-400"
            >
              Cancelar
            </Button>
            <Button
              onClick={handleDanger}
              disabled={actionBusy}
              className={cn(
                "text-xs",
                dangerDialog === "factory"
                  ? "bg-danger/20 border border-danger/40 text-danger hover:bg-danger/30"
                  : "bg-warn/10 border border-warn/40 text-warn hover:bg-warn/20"
              )}
            >
              {actionBusy ? "Aguarde..." : "Confirmar"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
