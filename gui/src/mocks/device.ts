import type { UseDeviceConfigReturn } from "@/hooks/useDeviceConfig";
import type { DeviceSnapshot } from "@/hooks/useDevicePolling";
import type { AxisConfig, ButtonConfig, DeviceConfig } from "@/types/protocol";
import { defaultDeviceConfig } from "@/types/protocol";

export const connectedSnapshot: DeviceSnapshot = {
  axes: { x: 12340, y: -8420, twist: 2600, unhealthy_mask: 0 },
  buttons: { mask: 0x00000015 },
  stats: {
    reports_sent: 124320,
    send_errors: 0,
    sensor_cycles: 124400,
    last_cycle_us: 420,
    max_cycle_us: 610,
  },
  connected: true,
  lastErrorMs: null,
};

export const disconnectedSnapshot: DeviceSnapshot = {
  axes: null,
  buttons: null,
  stats: null,
  connected: false,
  lastErrorMs: Date.now(),
};

export const unhealthyXAxisSnapshot: DeviceSnapshot = {
  ...connectedSnapshot,
  axes: { x: 0, y: -6100, twist: 18400, unhealthy_mask: 0b001 },
  stats: {
    ...connectedSnapshot.stats!,
    send_errors: 2,
    last_cycle_us: 730,
    max_cycle_us: 1410,
  },
};

export function makeDeviceConfig(overrides: Partial<DeviceConfig> = {}): DeviceConfig {
  const base = defaultDeviceConfig();
  return {
    ...base,
    ...overrides,
    axes: overrides.axes ?? base.axes,
    buttons: overrides.buttons ?? base.buttons,
  };
}

export function makeAxisConfig(partial: Partial<AxisConfig>): AxisConfig {
  return { ...defaultDeviceConfig().axes[0], ...partial };
}

export function makeButtonConfig(partial: Partial<ButtonConfig>): ButtonConfig {
  return { ...defaultDeviceConfig().buttons, ...partial };
}

export function makeDeviceConfigState(
  config = makeDeviceConfig(),
  overrides: Partial<UseDeviceConfigReturn> = {}
): UseDeviceConfigReturn {
  const state: UseDeviceConfigReturn = {
    config,
    dirty: false,
    loading: false,
    error: null,
    updateAxis: () => {},
    updateButtons: () => {},
    apply: async () => {},
    save: async () => {},
    reload: async () => {},
    reloadDefaults: async () => {},
  };

  return { ...state, ...overrides };
}
