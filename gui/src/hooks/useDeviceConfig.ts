/**
 * useDeviceConfig
 *
 * Loads DeviceConfig from firmware, tracks local edits, and provides
 * apply/save helpers. Dirty flag drives the "unsaved changes" banner.
 *
 * Usage:
 *   const { config, dirty, updateAxis, updateButtons, apply, save, reload } =
 *     useDeviceConfig();
 */

import { useCallback, useEffect, useState } from "react";
import {
  getConfig,
  setConfig,
  saveConfig,
  loadDefaults,
  DEVICE_CONFIG_EVENT,
  DEVICE_CONNECTION_EVENT,
} from "../lib/tauri";
import { withPollingSuspended } from "./useDevicePolling";
import type { AxisConfig, ButtonConfig, DeviceConfig } from "../types/protocol";
import { defaultDeviceConfig } from "../types/protocol";

export interface UseDeviceConfigReturn {
  config: DeviceConfig;
  dirty: boolean;
  loading: boolean;
  error: string | null;
  /** Replace one axis's config (0=X, 1=Y, 2=Twist). Marks dirty. */
  updateAxis: (index: 0 | 1 | 2, partial: Partial<AxisConfig>) => void;
  /** Replace button config. Marks dirty. */
  updateButtons: (partial: Partial<ButtonConfig>) => void;
  /** Send SetConfig to firmware (runtime only, not persisted). */
  apply: () => Promise<void>;
  /** Send SetConfig then SaveConfig (persists to flash). */
  save: () => Promise<void>;
  /** Reload from firmware (discards local changes). */
  reload: () => Promise<void>;
  /** Load firmware defaults into local state (does not send to device). */
  reloadDefaults: () => Promise<void>;
}

export function useDeviceConfig(): UseDeviceConfigReturn {
  const [config, setLocalConfig] = useState<DeviceConfig>(defaultDeviceConfig);
  const [dirty, setDirty] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const cfg = await getConfig();
      setLocalConfig(cfg);
      setDirty(false);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    const handleConnection = (event: Event) => {
      if ((event as CustomEvent<boolean>).detail) void reload();
    };
    const handleConfigChange = () => void reload();

    window.addEventListener(DEVICE_CONNECTION_EVENT, handleConnection);
    window.addEventListener(DEVICE_CONFIG_EVENT, handleConfigChange);
    void reload();

    return () => {
      window.removeEventListener(DEVICE_CONNECTION_EVENT, handleConnection);
      window.removeEventListener(DEVICE_CONFIG_EVENT, handleConfigChange);
    };
  }, [reload]);

  const updateAxis = useCallback(
    (index: 0 | 1 | 2, partial: Partial<AxisConfig>) => {
      const current = config.axes[index];
      const onlyEnabledChange = Object.keys(partial).every((key) => key === "enabled");

      if (!current.enabled && !onlyEnabledChange) return;

      setLocalConfig((prev) => {
        const axes = [...prev.axes] as DeviceConfig["axes"];
        axes[index] = { ...axes[index], ...partial };
        return { ...prev, axes };
      });
      setDirty(true);
    },
    [config]
  );

  const updateButtons = useCallback((partial: Partial<ButtonConfig>) => {
    setLocalConfig((prev) => ({
      ...prev,
      buttons: { ...prev.buttons, ...partial },
    }));
    setDirty(true);
  }, []);

  const apply = useCallback(async () => {
    setError(null);
    try {
      await withPollingSuspended(async () => {
        await setConfig(config);
      });
      setDirty(false);
    } catch (e) {
      setError(String(e));
      throw e;
    }
  }, [config]);

  const save = useCallback(async () => {
    setError(null);
    try {
      await withPollingSuspended(async () => {
        await setConfig(config);
        await saveConfig();
      });
      setDirty(false);
    } catch (e) {
      setError(String(e));
      throw e;
    }
  }, [config]);

  const reloadDefaults = useCallback(async () => {
    setError(null);
    try {
      await withPollingSuspended(async () => {
        await loadDefaults();
        await reload();
      });
    } catch (e) {
      setError(String(e));
      throw e;
    }
  }, [reload]);

  return {
    config,
    dirty,
    loading,
    error,
    updateAxis,
    updateButtons,
    apply,
    save,
    reload,
    reloadDefaults,
  };
}
