import type { Meta, StoryObj } from "@storybook/react";
import { Diagnostics } from "./Diagnostics";

const meta = {
  title: "Diagnostics/Diagnostics",
  component: Diagnostics,
  decorators: [
    (Story) => (
      <div className="h-[760px]">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof Diagnostics>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Healthy: Story = {
  args: {},
};

export const WithRuntimeWarnings: Story = {
  args: {},
};
