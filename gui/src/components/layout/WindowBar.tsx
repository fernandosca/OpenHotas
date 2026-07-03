import { Minus, X } from "lucide-react";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { cn } from "@/lib/utils";

interface Props {
  connected: boolean;
  screenLabel: string;
}

async function minimizeWindow() {
  if (isTauri()) {
    await getCurrentWindow().minimize();
  }
}

async function closeWindow() {
  if (isTauri()) {
    await getCurrentWindow().close();
  } else {
    window.close();
  }
}

async function startWindowDrag() {
  if (isTauri()) {
    await getCurrentWindow().startDragging();
  }
}

export function WindowBar({ connected, screenLabel }: Props) {
  return (
    <header
      data-tauri-drag-region
      onMouseDown={(event) => {
        if (event.button === 0 && !(event.target as HTMLElement).closest("button")) {
          void startWindowDrag();
        }
      }}
      className="flex h-8 flex-shrink-0 items-center justify-between bg-hud-bg pl-3 pr-2"
    >
      <div
        data-tauri-drag-region
        className="flex items-center gap-2 font-mono text-[10px] uppercase tracking-widest text-content-muted"
      >
        <span
          aria-hidden="true"
          className={cn(
            "h-1.5 w-1.5 rounded-full",
            connected ? "bg-ok" : "bg-danger"
          )}
        />
        <span data-tauri-drag-region>OpenHOTAS · {screenLabel}</span>
        <span className="sr-only">{connected ? "Dispositivo conectado" : "Dispositivo desconectado"}</span>
      </div>

      <div className="flex items-center">
        <button
          type="button"
          aria-label="Minimizar"
          onClick={() => void minimizeWindow()}
          className="flex h-6 w-8 items-center justify-center rounded text-content-muted hover:bg-hud-surface2 hover:text-content-primary"
        >
          <Minus className="h-4 w-4" strokeWidth={1.8} />
        </button>
        <button
          type="button"
          aria-label="Fechar"
          onClick={() => void closeWindow()}
          className="flex h-6 w-8 items-center justify-center rounded text-content-muted hover:bg-danger/15 hover:text-danger"
        >
          <X className="h-4 w-4" strokeWidth={1.8} />
        </button>
      </div>
    </header>
  );
}
