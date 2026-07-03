import type { ComponentPropsWithoutRef, CSSProperties } from "react";
import { TabsTrigger } from "@/components/ui/tabs";
import type { AxisId } from "@/types/protocol";
import { AXIS_COLORS, AXIS_RGB } from "@/theme/colors";
import { cn } from "@/lib/utils";

export const AXIS_IDS: readonly AxisId[] = ["X", "Y", "Twist"];

interface Props extends Omit<ComponentPropsWithoutRef<typeof TabsTrigger>, "value"> {
  axis: AxisId;
  enabled?: boolean;
  variant?: "solid" | "subtle";
}

export function AxisTabTrigger({
  axis,
  enabled = true,
  variant = "solid",
  className,
  style,
  children,
  ...props
}: Props) {
  const axisStyle = {
    "--axis-tab-color": AXIS_COLORS[axis],
    "--axis-tab-rgb": AXIS_RGB[axis],
    opacity: enabled ? 1 : 0.45,
    ...style,
  } as CSSProperties;

  return (
    <TabsTrigger
      value={axis}
      className={cn(
        "h-6 w-14 px-0 font-mono text-xs font-semibold",
        variant === "solid"
          ? "data-[state=active]:bg-[var(--axis-tab-color)] data-[state=active]:text-content-inverse"
          : "data-[state=active]:bg-[color:rgba(var(--axis-tab-rgb),0.12)] data-[state=active]:text-[var(--axis-tab-color)]",
        className,
      )}
      style={axisStyle}
      {...props}
    >
      {children ?? axis}
    </TabsTrigger>
  );
}
