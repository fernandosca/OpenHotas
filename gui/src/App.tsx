import { useState, type ReactNode } from "react";
import { Activity, Cable, ChartSpline, Joystick, Ruler, Settings as SettingsIcon, Unplug } from "lucide-react";
import { IconMatrix } from "@tabler/icons-react";
import { cn } from "@/lib/utils";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { AxesPage }    from "@/components/dashboard/Dashboard";
import { ButtonsPage } from "@/components/buttons/ButtonsPage";
import { Calibration } from "@/components/calibration/Calibration";
import { CurvePage }  from "@/components/curves/CurvePage";
import { Diagnostics } from "@/components/diagnostics/Diagnostics";
import { SettingsPage } from "@/components/settings/SettingsPage";
import { ConnectBar }  from "@/components/layout/ConnectBar";
import { WindowBar }  from "@/components/layout/WindowBar";
import { disconnect } from "@/lib/tauri";
import { useDevicePolling } from "@/hooks/useDevicePolling";
import { useDeviceConfig }  from "@/hooks/useDeviceConfig";
import openHotasLogo from "@/assets/openhotas-logo.png";

// ── nav items ─────────────────────────────────────────────────────────────────
type Screen = "axes" | "buttons" | "curves" | "calibration" | "diagnostics" | "settings";

const iconClass = "h-5 w-5";

const NAV: { id: Exclude<Screen, "settings">; label: string; icon: ReactNode }[] = [
  { id: "axes",        label: "Eixo",        icon: <Joystick className={iconClass} strokeWidth={1.8} /> },
  { id: "buttons",     label: "Botões",      icon: <IconMatrix className={iconClass} stroke={1.8} /> },
  { id: "curves",      label: "Curva",       icon: <ChartSpline className={iconClass} strokeWidth={1.8} /> },
  { id: "calibration", label: "Calibração",  icon: <Ruler className={iconClass} strokeWidth={1.8} /> },
  { id: "diagnostics", label: "Atividade",   icon: <Activity className={iconClass} strokeWidth={1.8} /> },
];

function DisconnectedState() {
  return (
    <div className="flex h-full items-center justify-center p-6">
      <div className="flex w-full max-w-md flex-col items-center gap-4 text-center">
        <div className="flex h-12 w-12 items-center justify-center rounded-lg border border-danger/30 bg-danger/10 font-mono text-lg text-danger">
          !
        </div>
        <div>
          <div className="text-sm font-medium text-slate-200">Conecte-se para iniciar</div>
          <div className="mt-1 text-xs text-slate-500">Selecione uma porta para conectar</div>
        </div>
        <ConnectBar connected={false} />
      </div>
    </div>
  );
}

export default function App() {
  const [screen, setScreen] = useState<Screen>("axes");
  const snapshot   = useDevicePolling();
  const deviceConfig = useDeviceConfig();

  return (
    <TooltipProvider delayDuration={400}>
      <div className="flex h-screen w-screen overflow-hidden bg-hud-bg select-none">

        {/* ── Sidebar ─────────────────────────────────────────── */}
        <aside className="
          flex flex-col items-center w-16 flex-shrink-0
          bg-hud-surface border-r border-hud-border2
          py-3 gap-1
        ">
          {/* Logo mark */}
          <div className="mb-2 flex h-9 w-9 items-center justify-center overflow-hidden rounded-md border border-cyan/30 bg-hud-surface2">
            <img
              src={openHotasLogo}
              alt="OpenHOTAS"
              className="h-full w-full object-cover"
              draggable={false}
            />
          </div>

          <div className="flex-1" />

          {/* Nav buttons */}
          <nav className="flex flex-col items-center gap-1">
            {NAV.map(({ id, label, icon }) => (
              <Tooltip key={id}>
                <TooltipTrigger asChild>
                  <button
                    onClick={() => setScreen(id)}
                    className={cn(
                      "relative w-11 h-11 rounded-lg flex items-center justify-center",
                      "text-lg transition-all duration-150",
                      screen === id
                        ? "bg-cyan-dim text-cyan"
                        : "text-slate-500 hover:bg-hud-surface2 hover:text-slate-300"
                    )}
                  >
                    {/* Active indicator */}
                    {screen === id && (
                      <span className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-6 bg-cyan rounded-r" />
                    )}
                    <span>{icon}</span>
                  </button>
                </TooltipTrigger>
                <TooltipContent side="right" className="bg-hud-surface2 border-hud-border2 text-slate-200 text-xs">
                  {label}
                </TooltipContent>
              </Tooltip>
            ))}
          </nav>

          <div className="flex-1" />

          {/* Settings */}
          <Tooltip>
            <TooltipTrigger asChild>
              <button
                type="button"
                onClick={() => setScreen("settings")}
                aria-label="Configurações"
                className={cn(
                  "relative mb-3 h-9 w-9 rounded-lg flex items-center justify-center text-base transition-all duration-150",
                  screen === "settings"
                    ? "bg-cyan-dim text-cyan"
                    : "text-slate-500 hover:bg-hud-surface2 hover:text-slate-300"
                )}
              >
                {screen === "settings" && (
                  <span className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-5 bg-cyan rounded-r" />
                )}
                <SettingsIcon className="h-5 w-5" strokeWidth={1.8} />
              </button>
            </TooltipTrigger>
            <TooltipContent side="right" className="bg-hud-surface2 border-hud-border2 text-slate-200 text-xs">
              Configurações
            </TooltipContent>
          </Tooltip>

          {/* Connection control */}
          <Tooltip>
            <TooltipTrigger asChild>
              <button
                type="button"
                onClick={() => { if (snapshot.connected) void disconnect(); }}
                aria-label={snapshot.connected ? "Desconectar dispositivo" : "Dispositivo desconectado"}
                className={cn(
                  "mb-2 h-9 w-9 rounded-lg border flex items-center justify-center transition-colors",
                  snapshot.connected
                    ? "border-ok/30 bg-ok/10 text-ok hover:border-danger/40 hover:bg-danger/10 hover:text-danger"
                    : "border-danger/30 bg-danger/10 text-danger"
                )}
              >
                {snapshot.connected ? (
                  <Cable className="h-5 w-5" strokeWidth={1.8} />
                ) : (
                  <Unplug className="h-5 w-5" strokeWidth={1.8} />
                )}
              </button>
            </TooltipTrigger>
            <TooltipContent side="right" className="bg-hud-surface2 border-hud-border2 text-slate-200 text-xs">
              {snapshot.connected ? "Conectado · clique para desconectar" : "Desconectado"}
            </TooltipContent>
          </Tooltip>
        </aside>

        {/* ── Main ────────────────────────────────────────────── */}
        <div className="flex flex-col flex-1 min-w-0">
          <WindowBar />

          {/* Screen content */}
          <main className="flex-1 overflow-auto scrollbar-thin animate-fade-in">
            {!snapshot.connected ? (
              <DisconnectedState />
            ) : (
              <>
                {screen === "axes"        && <AxesPage    snapshot={snapshot} deviceConfig={deviceConfig} />}
                {screen === "buttons"     && <ButtonsPage snapshot={snapshot} deviceConfig={deviceConfig} />}
                {screen === "curves"      && <CurvePage  deviceConfig={deviceConfig} />}
                {screen === "calibration" && <Calibration snapshot={snapshot} deviceConfig={deviceConfig} />}
                {screen === "diagnostics" && <Diagnostics />}
                {screen === "settings"    && <SettingsPage deviceConfig={deviceConfig} />}
              </>
            )}
          </main>
        </div>

      </div>
    </TooltipProvider>
  );
}
