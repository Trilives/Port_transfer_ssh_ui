import { cn } from "../lib/utils";
import { t } from "../i18n";
import type { Language, TunnelStatus } from "../types";

export function StatusBadge(props: { language: Language; status?: TunnelStatus }) {
  const running = props.status === "running";
  return (
    <span
      className={cn(
        "rounded-full px-2.5 py-1 text-xs font-medium",
        running
          ? "bg-emerald-100 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300"
          : "bg-slate-100 text-slate-500 dark:bg-slate-900 dark:text-slate-400",
      )}
    >
      {running ? t(props.language, "running") : t(props.language, "stopped")}
    </span>
  );
}
