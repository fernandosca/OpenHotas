import type { Meta, StoryObj } from "@storybook/react";
import { Badge } from "./badge";
import { Card, CardContent, CardHeader, CardTitle } from "./card";

const meta = {
  title: "UI/Card",
  component: Card,
  decorators: [
    (Story) => (
      <div className="max-w-md p-6">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof Card>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: () => (
    <Card>
      <CardHeader>
        <CardTitle>Runtime stats</CardTitle>
      </CardHeader>
      <CardContent className="space-y-2">
        <div className="flex justify-between text-sm">
          <span className="text-content-muted">Reports</span>
          <span className="font-mono text-content-primary">124,320</span>
        </div>
        <div className="flex justify-between text-sm">
          <span className="text-content-muted">Cycle</span>
          <span className="font-mono text-cyan">420 us</span>
        </div>
      </CardContent>
    </Card>
  ),
};

export const HudSurface: Story = {
  render: () => (
    <Card className="bg-hud-surface border-hud-border2">
      <CardHeader className="px-4 pt-3 pb-2">
        <CardTitle className="text-[11px] uppercase tracking-widest text-content-muted">
          Saúde dos sensores
        </CardTitle>
      </CardHeader>
      <CardContent className="px-4 pb-4 flex gap-2">
        <Badge variant="outline" className="border-ok/40 text-ok bg-ok/10">
          X OK
        </Badge>
        <Badge variant="outline" className="border-ok/40 text-ok bg-ok/10">
          Y OK
        </Badge>
        <Badge variant="outline" className="border-danger/40 text-danger bg-danger/10">
          Twist Fault
        </Badge>
      </CardContent>
    </Card>
  ),
};
