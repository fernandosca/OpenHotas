import { useState, useEffect } from "react";
import { Card, CardContent } from "@/components/ui/card";
import { getErrorCounters, getSensorStatus, getRuntimeStats } from "@/lib/tauri";
import type { ErrorCounters, SensorStatusReport, RuntimeStats } from "@/types/protocol";
import { cn } from "@/lib/utils";

export function Diagnostics() {
  const [counters,   setCounters]   = useState<ErrorCounters   | null>(null);
  const [sensorSt,  setSensorSt]    = useState<SensorStatusReport | null>(null);
  const [stats,     setStats]       = useState<RuntimeStats     | null>(null);
  const [refreshing, setRefreshing] = useState(false);

  async function refresh() {
    setRefreshing(true);
    try {
      const [ec, ss, rs] = await Promise.all([
        getErrorCounters(),
        getSensorStatus(),
        getRuntimeStats(),
      ]);
      setCounters(ec); setSensorSt(ss); setStats(rs);
    } catch (e) {
      console.warn("Diagnostics refresh failed", e);
    } finally {
      setRefreshing(false);
    }
  }

  useEffect(() => { refresh(); }, []);

  return (
    <div className="flex h-full flex-col p-4">
      <Card className="flex min-h-0 flex-1 flex-col bg-hud-surface border-hud-border2">
        <CardContent className="flex min-h-0 flex-1 flex-col px-4 py-4">
          <div>
            <div className="mb-2 text-[11px] uppercase tracking-widest text-content-muted">
              Runtime stats
            </div>
            <div className="space-y-2">
            {[
              ...(stats ? [
                ["Reports",       stats.reports_sent.toLocaleString()],
                ["Send errors",   stats.send_errors.toString()],
                ["Sensor cycles", stats.sensor_cycles.toLocaleString()],
                ["Last cycle",    `${stats.last_cycle_us} μs`],
                ["Max cycle",     `${stats.max_cycle_us} μs`],
              ] : []),
              ...(counters ? [
                ["Proto CRC",  counters.protocol_crc_errors.toString()],
                ["Sensor CRC", counters.sensor_crc_errors.toString()],
                ["Magneto",    counters.magnet_errors.toString()],
                ["Flash",      counters.flash_errors.toString()],
              ] : []),
              ...(sensorSt ? [
                ["Sensor X",     `${sensorSt.x.error_count}${sensorSt.x.healthy ? "" : " · FAULT"}`],
                ["Sensor Y",     `${sensorSt.y.error_count}${sensorSt.y.healthy ? "" : " · FAULT"}`],
                ["Sensor Twist", `${sensorSt.twist.error_count}${sensorSt.twist.healthy ? "" : " · FAULT"}`],
              ] : []),
            ].map(([label, value]) => (
              <div key={label} className="flex justify-between items-center">
                <span className="text-[10px] text-content-muted">{label}</span>
                <span className={cn(
                  "font-mono text-[11px]",
                  ["Proto CRC", "Sensor CRC", "Magneto", "Flash", "Send errors", "Sensor X", "Sensor Y", "Sensor Twist"].includes(label)
                    ? Number(value) === 0 ? "text-ok" : "text-warn"
                    : "text-content-primary"
                )}>
                  {value}
                </span>
              </div>
            ))}
            {!stats && !counters && !sensorSt && (
              <div className="text-xs text-content-dim text-center py-2">—</div>
            )}
            </div>

            <div className="mt-2 flex justify-end">
              <button
                type="button"
                onClick={refresh}
                disabled={refreshing}
                className="text-[11px] font-mono text-cyan hover:text-cyan/80 disabled:text-content-dim"
              >
                {refreshing ? "Atualizando..." : "Atualizar tudo"}
              </button>
            </div>
          </div>

        </CardContent>
      </Card>
    </div>
  );
}
