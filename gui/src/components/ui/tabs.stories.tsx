import type { Meta, StoryObj } from "@storybook/react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "./tabs";

const meta = {
  title: "UI/Tabs",
  component: Tabs,
  decorators: [
    (Story) => (
      <div className="max-w-md p-6">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof Tabs>;

export default meta;
type Story = StoryObj<typeof meta>;

export const AxisTabs: Story = {
  render: () => (
    <Tabs defaultValue="X">
      <TabsList className="bg-hud-surface2 border border-hud-border2">
        <TabsTrigger value="X" className="w-14 px-0">X</TabsTrigger>
        <TabsTrigger value="Y" className="w-14 px-0">Y</TabsTrigger>
        <TabsTrigger value="Twist" className="w-14 px-0">Twist</TabsTrigger>
      </TabsList>
      <TabsContent value="X" className="text-sm text-slate-300">
        Configuração do eixo X.
      </TabsContent>
      <TabsContent value="Y" className="text-sm text-slate-300">
        Configuração do eixo Y.
      </TabsContent>
      <TabsContent value="Twist" className="text-sm text-slate-300">
        Configuração do eixo Twist.
      </TabsContent>
    </Tabs>
  ),
};
