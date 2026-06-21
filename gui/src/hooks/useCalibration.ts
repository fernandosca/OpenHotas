/**
 * useCalibration
 *
 * State machine for the 4-step calibration flow:
 *   idle → session_open → min_captured → center_captured → complete
 *
 * Maps to firmware commands:
 *   StartCalibration → CaptureCalibrationPoint(Min) →
 *   CaptureCalibrationPoint(Center) → CaptureCalibrationPoint(Max) →
 *   FinishCalibration → (SaveConfig optional)
 */

import { useCallback, useState } from "react";
import {
  startCalibration,
  captureCalibrationPoint,
  finishCalibration,
  saveConfig,
} from "../lib/tauri";
import type { AxisId } from "../types/protocol";

export type CalStep =
  | "idle"
  | "session_open"
  | "min_captured"
  | "center_captured"
  | "complete";

export interface CapturedPoints {
  min: number | null;
  center: number | null;
  max: number | null;
}

export interface UseCalibrationReturn {
  step: CalStep;
  captures: CapturedPoints;
  persisted: boolean;
  busy: boolean;
  error: string | null;
  /** Open calibration session. axis can change between calls. */
  start: (axis: AxisId) => Promise<void>;
  /** Capture Min — call with axis at minimum position. */
  captureMin: (axis: AxisId, rawValue: number) => Promise<void>;
  /** Capture Center — call with axis at rest/center. */
  captureCenter: (axis: AxisId, rawValue: number) => Promise<void>;
  /** Capture Max and finalize runtime calibration. */
  captureMax: (axis: AxisId, rawValue: number) => Promise<void>;
  /** Finalize calibration (applies to runtime config, NOT flash). */
  finish: (axis: AxisId) => Promise<void>;
  /** Persist to flash after finish(). */
  persist: () => Promise<void>;
  /** Reset local state without touching firmware. */
  reset: () => void;
}

export function useCalibration(): UseCalibrationReturn {
  const [step, setStep] = useState<CalStep>("idle");
  const [captures, setCaptures] = useState<CapturedPoints>({
    min: null,
    center: null,
    max: null,
  });
  const [persisted, setPersisted] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const wrap = useCallback(async (fn: () => Promise<void>) => {
    setBusy(true);
    setError(null);
    try {
      await fn();
    } catch (e) {
      setError(String(e));
      throw e;
    } finally {
      setBusy(false);
    }
  }, []);

  const start = useCallback(
    (axis: AxisId) =>
      wrap(async () => {
        await startCalibration(axis);
        setStep("session_open");
        setCaptures({ min: null, center: null, max: null });
        setPersisted(false);
      }),
    [wrap]
  );

  const captureMin = useCallback(
    (axis: AxisId, rawValue: number) =>
      wrap(async () => {
        await captureCalibrationPoint(axis, "Min");
        setCaptures((p) => ({ ...p, min: rawValue }));
        setStep("min_captured");
      }),
    [wrap]
  );

  const captureCenter = useCallback(
    (axis: AxisId, rawValue: number) =>
      wrap(async () => {
        await captureCalibrationPoint(axis, "Center");
        setCaptures((p) => ({ ...p, center: rawValue }));
        setStep("center_captured");
      }),
    [wrap]
  );

  const captureMax = useCallback(
    (axis: AxisId, rawValue: number) =>
      wrap(async () => {
        await captureCalibrationPoint(axis, "Max");
        await finishCalibration(axis);
        setCaptures((p) => ({ ...p, max: rawValue }));
        setStep("complete");
      }),
    [wrap]
  );

  const finish = useCallback(
    (axis: AxisId) =>
      wrap(async () => {
        await finishCalibration(axis);
        setStep("complete");
        setPersisted(false);
      }),
    [wrap]
  );

  const persist = useCallback(
    () =>
      wrap(async () => {
        await saveConfig();
        setPersisted(true);
      }),
    [wrap]
  );

  const reset = useCallback(() => {
    setStep("idle");
    setCaptures({ min: null, center: null, max: null });
    setPersisted(false);
    setError(null);
    setBusy(false);
  }, []);

  return {
    step,
    captures,
    persisted,
    busy,
    error,
    start,
    captureMin,
    captureCenter,
    captureMax,
    finish,
    persist,
    reset,
  };
}
