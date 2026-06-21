import type { Meta, StoryObj } from "@storybook/react";
import { AxesPage } from "./Dashboard";
import {
  connectedSnapshot,
  disconnectedSnapshot,
  makeDeviceConfigState,
  unhealthyXAxisSnapshot,
} from "@/mocks/device";

const meta = {
  title: "Axes/AxesPage",
  component: AxesPage,
  decorators: [
    (Story) => (
      <div className="h-[760px]">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof AxesPage>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Connected: Story = {
  args: { snapshot: connectedSnapshot, deviceConfig: makeDeviceConfigState() },
};

export const UnhealthyAxis: Story = {
  args: { snapshot: unhealthyXAxisSnapshot, deviceConfig: makeDeviceConfigState() },
};

export const Disconnected: Story = {
  args: { snapshot: disconnectedSnapshot, deviceConfig: makeDeviceConfigState() },
};
