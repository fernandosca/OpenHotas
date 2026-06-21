import type { Meta, StoryObj } from "@storybook/react";
import { Calibration } from "./Calibration";
import {
  connectedSnapshot,
  disconnectedSnapshot,
  unhealthyXAxisSnapshot,
  makeAxisConfig,
  makeDeviceConfig,
  makeDeviceConfigState,
} from "@/mocks/device";


const enabledConfig = makeDeviceConfig();
const disabledXConfig = makeDeviceConfig({
  axes: [
    makeAxisConfig({ enabled: false }),
    makeAxisConfig({}),
    makeAxisConfig({ reset_ema_on_dz: true }),
  ],
});

const meta = {
  title: "Calibration/Calibration",
  component: Calibration,
  decorators: [
    (Story) => (
      <div className="h-[760px]">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof Calibration>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Connected: Story = {
  args: { snapshot: connectedSnapshot, deviceConfig: makeDeviceConfigState(enabledConfig) },
};

export const UnhealthyAxis: Story = {
  args: { snapshot: unhealthyXAxisSnapshot, deviceConfig: makeDeviceConfigState(enabledConfig) },
};

export const Disconnected: Story = {
  args: { snapshot: disconnectedSnapshot, deviceConfig: makeDeviceConfigState(enabledConfig) },
};

export const AxisDisabled: Story = {
  args: { snapshot: connectedSnapshot, deviceConfig: makeDeviceConfigState(disabledXConfig) },
};
