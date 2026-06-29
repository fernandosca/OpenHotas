import type { Meta, StoryObj } from "@storybook/react";
import { Alert, AlertDescription } from "./alert";

const meta = {
  title: "UI/Alert",
  component: Alert,
  decorators: [
    (Story) => (
      <div className="max-w-xl p-6">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof Alert>;

export default meta;
type Story = StoryObj<typeof meta>;

export const States: Story = {
  render: () => (
    <div className="space-y-3">
      <Alert className="bg-hud-surface2 border-hud-border2">
        <AlertDescription className="text-content-primary">
          Sessão de calibração aberta.
        </AlertDescription>
      </Alert>
      <Alert className="bg-warn/10 border-warn/40">
        <AlertDescription className="text-warn">
          Alterações não salvas no flash.
        </AlertDescription>
      </Alert>
      <Alert className="bg-danger/10 border-danger/40">
        <AlertDescription className="text-danger">
          Timeout aguardando resposta do firmware.
        </AlertDescription>
      </Alert>
    </div>
  ),
};
