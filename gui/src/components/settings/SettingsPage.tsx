import { useState } from "react";
import { confirm, open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import {
  Dialog, DialogContent, DialogHeader, DialogTitle,
  DialogDescription, DialogFooter,
} from "@/components/ui/dialog";
import type { UseDeviceConfigReturn } from "@/hooks/useDeviceConfig";
import { factoryReset, installFirmware, reboot } from "@/lib/tauri";
import { cn } from "@/lib/utils";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useTheme } from "@/theme/ThemeProvider";
import { THEMES, type ThemeId } from "@/theme/themes";

interface Props {
  deviceConfig: UseDeviceConfigReturn;
}

type DangerAction = "reboot" | "defaults" | "factory";
type UpdateState = "idle" | "ready" | "installing" | "done" | "error";

export function SettingsPage({ deviceConfig }: Props) {
  const { theme, setTheme } = useTheme();
  const [dangerDialog, setDangerDialog] = useState<DangerAction | null>(null);
  const [actionBusy, setActionBusy] = useState(false);
  const [uf2Path, setUf2Path] = useState<string | null>(null);
  const [updateState, setUpdateState] = useState<UpdateState>("idle");
  const [updateMessage, setUpdateMessage] = useState("");

  async function chooseFirmware() {
    const selected = await open({ multiple: false, filters: [{ name: "UF2 firmware", extensions: ["uf2"] }] });
    if (typeof selected === "string") {
      setUf2Path(selected);
      setUpdateState("ready");
      setUpdateMessage(selected.split(/[\\/]/).pop() ?? selected);
    }
  }

  async function updateFirmware() {
    if (!uf2Path) return;
    const approved = await confirm(
      "O dispositivo será reiniciado em modo bootloader e o firmware selecionado será gravado. Continuar?",
      { title: "Atualizar firmware", kind: "warning" }
    );
    if (!approved) return;
    setUpdateState("installing");
    setUpdateMessage("Reiniciando em modo bootloader e aguardando o volume RPI-RP2...");
    try {
      const result = await installFirmware(uf2Path);
      setUpdateState("done");
      setUpdateMessage(`Firmware copiado (${result.bytes_copied} bytes). O dispositivo será reiniciado.`);
    } catch (error) {
      setUpdateState("error");
      setUpdateMessage(String(error));
    }
  }

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
          <div className="mb-3 text-[11px] uppercase tracking-widest text-content-muted">
            Configurações
          </div>

          <div className="border-b border-hud-border pb-4">
            <div className="mb-2 text-[10px] uppercase tracking-widest text-content-muted">
              Aparência
            </div>
            <div className="flex items-center justify-between gap-4">
              <div>
                <div className="text-xs text-content-primary">Tema da interface</div>
                <div className="mt-0.5 text-[10px] text-content-muted">
                  {THEMES.find((item) => item.id === theme)?.description}
                </div>
              </div>
              <Select value={theme} onValueChange={(value) => setTheme(value as ThemeId)}>
                <SelectTrigger className="h-8 w-40 border-hud-border2 bg-hud-surface2 text-xs text-content-primary">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent className="border-hud-border2 bg-hud-raised text-content-primary">
                  {THEMES.map((item) => (
                    <SelectItem key={item.id} value={item.id} className="text-xs">
                      {item.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>

          <div className="mt-auto border-t border-hud-border pt-3">
            <div className="mb-2 text-[10px] uppercase tracking-widest text-content-muted">
              Atualização de firmware
            </div>
            <div className="mb-3 flex flex-col gap-2">
              <Button variant="outline" size="sm" onClick={chooseFirmware} disabled={updateState === "installing"}
                className="h-8 text-xs border-hud-border2 text-content-muted">
                Selecionar arquivo .uf2
              </Button>
              {uf2Path && (
                <Button size="sm" onClick={updateFirmware} disabled={updateState === "installing"}
                  className="h-8 text-xs bg-warn/10 border border-warn/40 text-warn hover:bg-warn/20">
                  {updateState === "installing" ? "Atualizando..." : "Instalar firmware"}
                </Button>
              )}
              {updateMessage && (
                <div className={cn("break-all text-[10px]", updateState === "error" ? "text-danger" : updateState === "done" ? "text-success" : "text-content-muted")}>
                  {updateMessage}
                </div>
              )}
              <div className="text-[10px] text-content-muted">
                O dispositivo desconectará durante a gravação. Não remova o cabo USB.
              </div>
            </div>

            <div className="border-t border-hud-border pt-3">
            <div className="mb-2 text-[10px] uppercase tracking-widest text-danger/70">
              Ações do dispositivo
            </div>
            <div className="flex flex-col gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setDangerDialog("reboot")}
                className="h-8 text-xs border-hud-border2 text-content-muted hover:border-warn/40 hover:text-warn"
              >
                ↺ Reboot
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setDangerDialog("defaults")}
                className="h-8 text-xs border-hud-border2 text-content-muted hover:border-warn/40 hover:text-warn"
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
          </div>
        </CardContent>
      </Card>

      <Dialog open={dangerDialog !== null} onOpenChange={(open) => !open && setDangerDialog(null)}>
        <DialogContent className="bg-hud-surface border-hud-border2 text-content-primary">
          <DialogHeader>
            <DialogTitle>
              {dangerDialog === "reboot" && "Reboot do dispositivo?"}
              {dangerDialog === "defaults" && "Carregar defaults?"}
              {dangerDialog === "factory" && "Factory reset?"}
            </DialogTitle>
            <DialogDescription className="text-content-muted text-xs">
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
              className="text-content-muted"
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
