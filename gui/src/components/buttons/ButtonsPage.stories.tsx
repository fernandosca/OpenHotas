import type { Meta, StoryObj } from "@storybook/react";
import { ButtonsPage } from "./ButtonsPage";
import {
  connectedSnapshot,
  disconnectedSnapshot,
  makeDeviceConfigState,
  unhealthyXAxisSnapshot,
} from "@/mocks/device";

const meta = {
  title: "Buttons/ButtonsPage",
  component: ButtonsPage,
  decorators: [
    (Story) => (
      <div className="h-[760px]">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof ButtonsPage>;

export default meta;
type Story = StoryObj<typeof meta>;

export const ActiveButtons: Story = {
  args: { snapshot: connectedSnapshot, deviceConfig: makeDeviceConfigState() },
};

export const ManyPressed: Story = {
  args: {
    snapshot: {
      ...connectedSnapshot,
      buttons: { mask: 0b1010_0001_0000_1111_0101_0000_0011_1011 },
    },
    deviceConfig: makeDeviceConfigState(),
  },
};

export const RuntimeWarnings: Story = {
  args: { snapshot: unhealthyXAxisSnapshot, deviceConfig: makeDeviceConfigState() },
};

export const Disconnected: Story = {
  args: { snapshot: disconnectedSnapshot, deviceConfig: makeDeviceConfigState() },
};
