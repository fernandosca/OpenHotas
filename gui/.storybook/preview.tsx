import type { Preview } from "@storybook/react";
import { TooltipProvider } from "../src/components/ui/tooltip";
import "../src/index.css";

const preview: Preview = {
  decorators: [
    (Story) => (
      <TooltipProvider delayDuration={0}>
        <div className="min-h-screen bg-hud-bg text-slate-200">
          <Story />
        </div>
      </TooltipProvider>
    ),
  ],
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
    layout: "fullscreen",
  },
};

export default preview;
