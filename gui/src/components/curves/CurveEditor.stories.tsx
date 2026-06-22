import type { Meta, StoryObj } from "@storybook/react";
import { CurveEditor } from "./CurveEditor";

type CurveArgs = React.ComponentProps<typeof CurveEditor>;

function CurvePreview(args: CurveArgs) {
  return (
    <div className="max-w-xl p-4">
      <CurveEditor {...args} />
    </div>
  );
}

const meta = {
  title: "Curves/CurveEditor",
  component: CurveEditor,
  render: (args) => <CurvePreview {...args} />,
} satisfies Meta<typeof CurveEditor>;

export default meta;
type Story = StoryObj<typeof meta>;

export const XAxis: Story = {
  args: {
    axisIndex: 0,
    responseCurve: {
      point_left: { x: -500, y: -500 },
      point_right: { x: 500, y: 500 },
    },
    deadzonePermille: 20,
  },
};

export const Twist: Story = {
  args: {
    axisIndex: 2,
    responseCurve: {
      point_left: { x: -250, y: -600 },
      point_right: { x: 250, y: 600 },
    },
    deadzonePermille: 45,
  },
};

export const Disabled: Story = {
  args: {
    axisIndex: 0,
    responseCurve: {
      point_left: { x: -500, y: -500 },
      point_right: { x: 500, y: 500 },
    },
    deadzonePermille: 20,
    disabled: true,
  },
};
