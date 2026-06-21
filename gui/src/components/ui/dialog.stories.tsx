import type { Meta, StoryObj } from "@storybook/react";
import { Button } from "./button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "./dialog";

const meta = {
  title: "UI/Dialog",
  component: Dialog,
  decorators: [
    (Story) => (
      <div className="p-6">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof Dialog>;

export default meta;
type Story = StoryObj<typeof meta>;

export const ConfirmFactoryReset: Story = {
  render: () => (
    <Dialog defaultOpen>
      <DialogContent className="bg-hud-surface border-hud-border2 text-slate-200">
        <DialogHeader>
          <DialogTitle>Factory reset?</DialogTitle>
          <DialogDescription className="text-slate-400">
            Apaga configuração e calibração do flash. Esta ação não pode ser desfeita.
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="ghost" className="text-slate-400">
            Cancelar
          </Button>
          <Button className="bg-danger/20 border border-danger/40 text-danger hover:bg-danger/30">
            Confirmar
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  ),
};
