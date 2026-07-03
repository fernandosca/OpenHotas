import { useState, useEffect } from "react";
import { Card, CardContent } from "@/components/ui/card";
import { getErrorCounters, getSensorStatus, getRuntimeStats } from "@/lib/tauri";
import type { ErrorCounters, SensorStatusReport, RuntimeStats } from "@/types/protocol";
import { cn } from "@/lib/utils";

interface DiagnosticRowProps {
  label: string;
  value: string;
  status?: "ok" | "warning";
}

function DiagnosticRow({ label, value, status }: DiagnosticRowProps) {
  return (
    <div className="flex items-center justify-between gap-3">
      <span className="text-[10px] text-content-muted">{label}</span>
      <span className={cn(
        "font-mono text-[11px]",
        status === "ok" ? "text-ok" : status === "warning" ? "text-warn" : "text-content-primary",
      )}>
        {value}
      </span>
    </div>
  );
}

function DiagnosticSection({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="rounded-md border border-hud-border bg-hud-surface2/40 p-3">
      <h2 className="mb-3 text-[10px] uppercase tracking-widest text-content-muted">{title}</h2>
      <div className="space-y-2">{children}</div>
    </section>
  );
}

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
      <Card className="mx-auto flex min-h-0 w-full max-w-6xl flex-1 flex-col bg-hud-surface border-hud-border2">
        <CardContent className="flex min-h-0 flex-1 flex-col px-4 py-4">
          <div>
            <div className="grid gap-3 md:grid-cols-3">
              <DiagnosticSection title="Runtime">
                {stats ? (
                  <>
                    <DiagnosticRow label="Reports" value={stats.reports_sent.toLocaleString()} />
                    <DiagnosticRow label="Send errors" value={stats.send_errors.toString()}
                      status={stats.send_errors === 0 ? "ok" : "warning"} />
                    <DiagnosticRow label="Sensor cycles" value={stats.sensor_cycles.toLocaleString()} />
                    <DiagnosticRow label="Last cycle" value={`${stats.last_cycle_us} μs`} />
                    <DiagnosticRow label="Max cycle" value={`${stats.max_cycle_us} μs`} />
                  </>
                ) : <div className="py-2 text-center text-xs text-content-dim">—</div>}
              </DiagnosticSection>

              <DiagnosticSection title="Erros">
                {counters ? (
                  <>
                    <DiagnosticRow label="Proto CRC" value={counters.protocol_crc_errors.toString()}
                      status={counters.protocol_crc_errors === 0 ? "ok" : "warning"} />
                    <DiagnosticRow label="Sensor CRC" value={counters.sensor_crc_errors.toString()}
                      status={counters.sensor_crc_errors === 0 ? "ok" : "warning"} />
                    <DiagnosticRow label="Magneto" value={counters.magnet_errors.toString()}
                      status={counters.magnet_errors === 0 ? "ok" : "warning"} />
                    <DiagnosticRow label="Flash" value={counters.flash_errors.toString()}
                      status={counters.flash_errors === 0 ? "ok" : "warning"} />
                  </>
                ) : <div className="py-2 text-center text-xs text-content-dim">—</div>}
              </DiagnosticSection>

              <DiagnosticSection title="Sensores">
                {sensorSt ? (
                  <>
                    {(["x", "y", "twist"] as const).map((sensor) => {
                      const report = sensorSt[sensor];
                      const label = sensor === "twist" ? "Sensor Twist" : `Sensor ${sensor.toUpperCase()}`;
                      return (
                        <DiagnosticRow key={sensor} label={label}
                          value={`${report.error_count}${report.healthy ? "" : " · FAULT"}`}
                          status={report.healthy && report.error_count === 0 ? "ok" : "warning"} />
                      );
                    })}
                  </>
                ) : <div className="py-2 text-center text-xs text-content-dim">—</div>}
              </DiagnosticSection>
            </div>

            <div className="mt-3 flex justify-end">
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
