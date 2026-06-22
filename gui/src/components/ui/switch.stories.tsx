import { useState } from "react";
import type { Meta, StoryObj } from "@storybook/react";
import { Label } from "./label";
import { Switch } from "./switch";

function ControlledSwitch() {
  const [checked, setChecked] = useState(true);
  return (
    <div className="flex items-center gap-3">
      <Switch id="axis-enabled" checked={checked} onCheckedChange={setChecked} />
      <Label htmlFor="axis-enabled">Eixo habilitado</Label>
    </div>
  );
}

const meta = {
  title: "UI/Switch",
  component: Switch,
  decorators: [
    (Story) => (
      <div className="p-6">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof Switch>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Enabled: Story = {
  render: () => <ControlledSwitch />,
};

export const Disabled: Story = {
  render: () => <Switch disabled checked />,
};
