import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from "@/components/ui/select";
import { listPorts, connect, type PortInfo } from "@/lib/tauri";

interface Props { connected: boolean }

const LAST_PORT_KEY = "openhotas.lastPort";

export function ConnectBar({ connected }: Props) {
  const [ports, setPorts]       = useState<PortInfo[]>([]);
  const [selected, setSelected] = useState<string>(() => localStorage.getItem(LAST_PORT_KEY) ?? "");
  const [busy, setBusy]         = useState(false);
  const [error, setError]       = useState<string | null>(null);

  async function refresh() {
    try {
      const p = await listPorts();
      setPorts(p);
      if (p.length && !selected) {
        const lastPort = localStorage.getItem(LAST_PORT_KEY);
        const nextPort = lastPort && p.some((port) => port.name === lastPort) ? lastPort : p[0].name;
        setSelected(nextPort);
      }
    } catch { /* no ports available */ }
  }

  useEffect(() => { refresh(); }, []);

  async function handleConnect() {
    if (!selected) return;
    setBusy(true); setError(null);
    try {
      await connect(selected);
      localStorage.setItem(LAST_PORT_KEY, selected);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="flex w-full max-w-md flex-col items-center gap-3">
      {!connected && (
        <div className="flex w-full items-center justify-center gap-2">
          <Select value={selected} onValueChange={setSelected}>
            <SelectTrigger className="
              h-8 flex-1 text-xs font-mono
              bg-hud-surface2 border-hud-border2
              text-slate-300 focus:ring-cyan/30
            ">
              <SelectValue placeholder="Selecionar porta…" />
            </SelectTrigger>
            <SelectContent className="bg-hud-surface2 border-hud-border2 text-slate-200">
              {ports.map((p) => (
                <SelectItem key={p.name} value={p.name} className="text-xs font-mono focus:bg-cyan-dim focus:text-cyan">
                  {p.name}{p.description ? ` — ${p.description}` : ""}
                </SelectItem>
              ))}
              {ports.length === 0 && (
                <div className="px-2 py-1.5 text-xs text-slate-500">
                  Nenhuma porta encontrada
                </div>
              )}
            </SelectContent>
          </Select>

          <Button
            size="sm"
            variant="outline"
            className="h-8 text-xs px-3 bg-transparent border-hud-border2 text-slate-400 hover:text-slate-200 hover:bg-hud-surface2"
            onClick={refresh}
          >
            ↻
          </Button>

          <Button
            size="sm"
            disabled={!selected || busy}
            onClick={handleConnect}
            className="h-8 text-xs px-3 bg-cyan-dim border border-cyan/30 text-cyan hover:bg-cyan/20"
          >
            {busy ? "Conectando…" : "Conectar"}
          </Button>
        </div>
      )}

      {error && (
        <span className="max-w-full truncate text-[10px] font-mono text-danger">
          {error}
        </span>
      )}
    </div>
  );
}
