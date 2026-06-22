import type { Meta, StoryObj } from "@storybook/react";
import { ConnectBar } from "./ConnectBar";

const meta = {
  title: "Layout/ConnectBar",
  component: ConnectBar,
} satisfies Meta<typeof ConnectBar>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Disconnected: Story = {
  args: { connected: false },
};

export const Connected: Story = {
  args: { connected: true },
};
