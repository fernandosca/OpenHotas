import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface Props {
  dirty: boolean;
  loading: boolean;
  error?: string | null;
  onDiscard: () => void;
  onSave: () => void;
  className?: string;
  compact?: boolean;
}

export function UnsavedChangesBar({
  dirty, loading, error, onDiscard, onSave, className, compact = false,
}: Props) {
  return (
    <div className={cn("space-y-3", className)}>
      {error && (
        <Alert className={cn("border-danger/40 bg-danger/10", compact ? "py-1.5" : "py-2")}>
          <AlertDescription className="text-xs text-danger">{error}</AlertDescription>
        </Alert>
      )}
      <Alert className={cn(
        compact ? "py-1.5" : "py-2",
        dirty ? "animate-fade-in border-warn/40 bg-warn/10" : "border-hud-border2 bg-hud-surface2",
      )}>
        <AlertDescription className="flex items-center justify-between gap-3">
          <span className={cn("text-xs", dirty ? "text-warn" : "text-content-muted")}>
            {dirty ? "Alterações não salvas no flash" : "Sem alterações pendentes"}
          </span>
          <div className="flex gap-2">
            <Button size="sm" variant="ghost" onClick={onDiscard} disabled={!dirty || loading}
              className="h-7 text-xs text-content-muted hover:text-content-primary disabled:opacity-40">
              Descartar
            </Button>
            <Button size="sm" onClick={onSave} disabled={!dirty || loading}
              className="h-7 border border-ok/30 bg-ok/10 text-xs text-ok hover:bg-ok/20 disabled:opacity-40">
              Salvar
            </Button>
          </div>
        </AlertDescription>
      </Alert>
    </div>
  );
}
