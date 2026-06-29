import { Minus, X } from "lucide-react";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

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

export function WindowBar() {
  return (
    <header className="flex h-8 flex-shrink-0 items-center justify-end border-b border-hud-border2 bg-hud-surface px-2">
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
    </header>
  );
}
