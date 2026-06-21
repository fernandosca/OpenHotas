import type { Meta, StoryObj } from "@storybook/react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./select";

const meta = {
  title: "UI/Select",
  component: Select,
  decorators: [
    (Story) => (
      <div className="max-w-xs p-6">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof Select>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Debounce: Story = {
  render: () => (
    <Select defaultValue="5">
      <SelectTrigger className="bg-hud-surface2 border-hud-border2 text-slate-300">
        <SelectValue placeholder="Debounce" />
      </SelectTrigger>
      <SelectContent className="bg-hud-surface2 border-hud-border2 text-slate-200">
        {[1, 2, 5, 10, 20].map((value) => (
          <SelectItem key={value} value={String(value)}>
            {value} ms
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  ),
};
