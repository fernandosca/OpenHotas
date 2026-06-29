import type { Meta, StoryObj } from "@storybook/react";
import { Input } from "./input";
import { Label } from "./label";

const meta = {
  title: "UI/Input",
  component: Input,
  decorators: [
    (Story) => (
      <div className="max-w-sm p-6">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof Input>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Text: Story = {
  args: {
    placeholder: "Nome do perfil",
  },
};

export const NumberField: Story = {
  render: () => (
    <div className="space-y-2">
      <Label htmlFor="max-jump">Max jump raw</Label>
      <Input
        id="max-jump"
        type="number"
        defaultValue={4915}
        className="font-mono text-right bg-hud-surface2 border-hud-border2 text-content-primary"
      />
    </div>
  ),
};

export const Disabled: Story = {
  args: {
    disabled: true,
    value: "Dispositivo desconectado",
  },
};
