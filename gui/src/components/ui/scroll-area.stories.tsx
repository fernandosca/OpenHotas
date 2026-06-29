import type { Meta, StoryObj } from "@storybook/react";
import { ScrollArea } from "./scroll-area";

const meta = {
  title: "UI/ScrollArea",
  component: ScrollArea,
  decorators: [
    (Story) => (
      <div className="max-w-md p-6">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof ScrollArea>;

export default meta;
type Story = StoryObj<typeof meta>;

export const ProtocolLog: Story = {
  render: () => (
    <ScrollArea className="h-40 rounded-md border border-hud-border2 bg-hud-surface p-3">
      <div className="space-y-1 pr-3 font-mono text-[10px]">
        {Array.from({ length: 24 }, (_, index) => (
          <div key={index} className="flex gap-2">
            <span className="text-content-dim">12:{String(index).padStart(2, "0")}:04</span>
            <span className={index % 7 === 0 ? "text-warn" : "text-ok"}>
              {index % 7 === 0 ? "CRC sensor: 1" : "Ack recebido"}
            </span>
          </div>
        ))}
      </div>
    </ScrollArea>
  ),
};
