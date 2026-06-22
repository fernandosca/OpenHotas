import { useState } from "react";
import type { Meta, StoryObj } from "@storybook/react";
import { Slider } from "./slider";

function RawLine({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex justify-between gap-6 rounded border border-hud-border bg-hud-surface2 px-3 py-2 font-mono text-[11px]">
      <span className="text-slate-500">{label}</span>
      <span className="text-cyan">{value}</span>
    </div>
  );
}

function ControlledSlider({ defaultValue = 30 }: { defaultValue?: number }) {
  const [value, setValue] = useState(defaultValue);
  return (
    <div className="max-w-md space-y-3">
      <div className="flex items-center gap-3">
        <Slider
          min={0}
          max={100}
          step={1}
          value={[value]}
          onValueChange={([next]) => setValue(next)}
          className="w-72"
        />
        <span className="w-12 text-right font-mono text-xs text-slate-300">{value}%</span>
      </div>
      <RawLine label="Radix value" value={`[${value}]`} />
    </div>
  );
}

function ProtocolSliderValues() {
  const [deadzonePermille, setDeadzonePermille] = useState(120);
  const [emaPermille, setEmaPermille] = useState(420);

  return (
    <div className="max-w-xl space-y-6">
      <div className="space-y-2">
        <div className="flex items-center justify-between text-xs">
          <span className="text-slate-300">Deadzone</span>
          <span className="font-mono text-slate-400">
            {(deadzonePermille / 10).toFixed(1)}%
          </span>
        </div>
        <Slider
          min={0}
          max={200}
          step={1}
          value={[deadzonePermille]}
          onValueChange={([next]) => setDeadzonePermille(next)}
        />
        <RawLine label="AxisConfig.deadzone_permille" value={String(deadzonePermille)} />
        <RawLine label="Radix value" value={`[${deadzonePermille}]`} />
      </div>

      <div className="space-y-2">
        <div className="flex items-center justify-between text-xs">
          <span className="text-slate-300">EMA alpha</span>
          <span className="font-mono text-slate-400">
            {(emaPermille / 1000).toFixed(3)}
          </span>
        </div>
        <Slider
          min={1}
          max={1000}
          step={1}
          value={[emaPermille]}
          onValueChange={([next]) => setEmaPermille(next)}
        />
        <RawLine label="AxisConfig.ema_permille" value={String(emaPermille)} />
        <RawLine label="Radix value" value={`[${emaPermille}]`} />
      </div>
    </div>
  );
}

const meta = {
  title: "UI/Slider",
  component: Slider,
  decorators: [
    (Story) => (
      <div className="p-6">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof Slider>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Deadzone: Story = {
  render: () => <ControlledSlider defaultValue={12} />,
};

export const ProtocolRawValues: Story = {
  render: () => <ProtocolSliderValues />,
};

export const Disabled: Story = {
  render: () => (
    <div className="flex items-center gap-3">
      <Slider disabled value={[45]} className="w-72" />
      <span className="w-12 text-right font-mono text-xs text-slate-500">45%</span>
    </div>
  ),
};
