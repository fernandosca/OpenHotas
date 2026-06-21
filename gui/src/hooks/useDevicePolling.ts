/**
 * useDevicePolling
 *
 * Polls the firmware for live data at ~60 Hz (processedAxes + buttonStates)
 * and ~4 Hz (runtimeStats). Returns the latest snapshot and a connection flag.
 *
 * Usage:
 *   const { axes, buttons, stats, connected } = useDevicePolling();
 */

import { useEffect, useRef, useState, useCallback } from "react";
import {
  getProcessedAxes,
  getButtonStates,
  getRuntimeStats,
} from "../lib/tauri";
import type { ProcessedAxes, ButtonStates, RuntimeStats } from "../types/protocol";

export interface DeviceSnapshot {
  axes: ProcessedAxes | null;
  buttons: ButtonStates | null;
  stats: RuntimeStats | null;
  connected: boolean;
  lastErrorMs: number | null;
}

const AXES_INTERVAL_MS = 16;   // ~60 Hz
const STATS_INTERVAL_MS = 250; // 4 Hz
let pollingSuspendCount = 0;

export async function withPollingSuspended<T>(operation: () => Promise<T>): Promise<T> {
  pollingSuspendCount++;
  try {
    return await operation();
  } finally {
    pollingSuspendCount = Math.max(0, pollingSuspendCount - 1);
  }
}

function pollingSuspended(): boolean {
  return pollingSuspendCount > 0;
}

export function useDevicePolling(): DeviceSnapshot {
  const [snapshot, setSnapshot] = useState<DeviceSnapshot>({
    axes: null,
    buttons: null,
    stats: null,
    connected: false,
    lastErrorMs: null,
  });

  const axesTimer = useRef<ReturnType<typeof setInterval> | null>(null);
  const statsTimer = useRef<ReturnType<typeof setInterval> | null>(null);
  const errorCount = useRef(0);
  const axesInFlight = useRef(false);
  const statsInFlight = useRef(false);

  const pollAxes = useCallback(async () => {
    if (pollingSuspended()) return;
    if (axesInFlight.current) return;
    axesInFlight.current = true;
    try {
      const axes = await getProcessedAxes();
      const buttons = await getButtonStates();
      errorCount.current = 0;
      setSnapshot((prev) => ({
        ...prev,
        axes,
        buttons,
        connected: true,
        lastErrorMs: null,
      }));
    } catch {
      errorCount.current++;
      if (errorCount.current >= 3) {
        setSnapshot((prev) => ({
          ...prev,
          connected: false,
          lastErrorMs: Date.now(),
        }));
      }
    } finally {
      axesInFlight.current = false;
    }
  }, []);

  const pollStats = useCallback(async () => {
    if (pollingSuspended()) return;
    if (statsInFlight.current) return;
    statsInFlight.current = true;
    try {
      const stats = await getRuntimeStats();
      setSnapshot((prev) => ({ ...prev, stats }));
    } catch {
      // non-fatal — stats missing is ok
    } finally {
      statsInFlight.current = false;
    }
  }, []);

  useEffect(() => {
    pollAxes();
    pollStats();
    axesTimer.current = setInterval(pollAxes, AXES_INTERVAL_MS);
    statsTimer.current = setInterval(pollStats, STATS_INTERVAL_MS);

    return () => {
      if (axesTimer.current) clearInterval(axesTimer.current);
      if (statsTimer.current) clearInterval(statsTimer.current);
    };
  }, [pollAxes, pollStats]);

  return snapshot;
}
