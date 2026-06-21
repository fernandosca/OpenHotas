import type { Meta, StoryObj } from "@storybook/react";
import { Badge } from "./badge";

const meta = {
  title: "UI/Badge",
  component: Badge,
  decorators: [
    (Story) => (
      <div className="p-6">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof Badge>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Variants: Story = {
  render: () => (
    <div className="flex flex-wrap gap-2">
      <Badge>Default</Badge>
      <Badge variant="secondary">Secondary</Badge>
      <Badge variant="outline">Outline</Badge>
      <Badge variant="destructive">Destructive</Badge>
    </div>
  ),
};

export const Status: Story = {
  render: () => (
    <div className="flex flex-wrap gap-2">
      <Badge variant="outline" className="border-ok/40 text-ok bg-ok/10">
        X OK
      </Badge>
      <Badge variant="outline" className="border-warn/40 text-warn bg-warn/10">
        BUSY
      </Badge>
      <Badge variant="outline" className="border-danger/50 text-danger bg-danger/10">
        FAULT
      </Badge>
      <Badge variant="outline" className="border-cyan/30 text-cyan bg-cyan-dim">
        v1.0
      </Badge>
    </div>
  ),
};
