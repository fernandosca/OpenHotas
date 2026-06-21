import type { Meta, StoryObj } from "@storybook/react";
import { CurvePage } from "./CurvePage";
import {
  makeAxisConfig,
  makeDeviceConfig,
  makeDeviceConfigState,
} from "@/mocks/device";

const tunedConfig = makeDeviceConfig({
  axes: [
    makeAxisConfig({
      deadzone_permille: 30,
      ema_permille: 420,
      response_curve: {
        point_left: { x: -400, y: -250 },
        point_right: { x: 400, y: 250 },
      },
    }),
    makeAxisConfig({
      inverted: true,
      deadzone_permille: 18,
      response_curve: {
        point_left: { x: -600, y: -400 },
        point_right: { x: 600, y: 400 },
      },
    }),
    makeAxisConfig({
      deadzone_permille: 45,
      reset_ema_on_dz: true,
      travel: { travel_limit_pct: 85 },
      response_curve: {
        point_left: { x: -250, y: -600 },
        point_right: { x: 250, y: 600 },
      },
    }),
  ],
});

const meta = {
  title: "Curves/CurvePage",
  component: CurvePage,
  decorators: [
    (Story) => (
      <div className="min-h-screen py-4">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof CurvePage>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    deviceConfig: makeDeviceConfigState(tunedConfig),
  },
};

export const Dirty: Story = {
  args: {
    deviceConfig: makeDeviceConfigState(tunedConfig, { dirty: true }),
  },
};

export const Error: Story = {
  args: {
    deviceConfig: makeDeviceConfigState(tunedConfig, {
      error: "Timeout aguardando resposta SetConfig",
    }),
  },
};

export const AxisDisabled: Story = {
  args: {
    deviceConfig: makeDeviceConfigState(
      makeDeviceConfig({
        axes: [
          makeAxisConfig({
            enabled: false,
            deadzone_permille: 30,
            response_curve: {
              point_left: { x: -400, y: -250 },
              point_right: { x: 400, y: 250 },
            },
          }),
          makeAxisConfig({
            inverted: true,
            deadzone_permille: 18,
            response_curve: {
              point_left: { x: -600, y: -400 },
              point_right: { x: 600, y: 400 },
            },
          }),
          makeAxisConfig({
            deadzone_permille: 45,
            reset_ema_on_dz: true,
            response_curve: {
              point_left: { x: -250, y: -600 },
              point_right: { x: 250, y: 600 },
            },
          }),
        ],
      }),
    ),
  },
};
