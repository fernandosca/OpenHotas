import type { Preview } from "@storybook/react";
import { TooltipProvider } from "../src/components/ui/tooltip";
import { ThemeProvider } from "../src/theme/ThemeProvider";
import "../src/index.css";

const preview: Preview = {
  decorators: [
    (Story) => (
      <ThemeProvider>
        <TooltipProvider delayDuration={0}>
          <div className="min-h-screen bg-hud-bg text-content-primary">
            <Story />
          </div>
        </TooltipProvider>
      </ThemeProvider>
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
