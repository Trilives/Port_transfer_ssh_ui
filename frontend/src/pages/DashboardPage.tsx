import { Activity, CircleSlash, Server } from "lucide-react";
import { Button } from "../components/ui/button";
import { Card, CardDescription, CardHeader, CardTitle } from "../components/ui/card";
import { cn } from "../lib/utils";
import { t } from "../i18n";
import type { Host, Language, LogEntry } from "../types";

const KEY_EVENT_MARKERS = ["connected", "disconnected", "cleaned up", "exited"];

function isKeyEvent(log: LogEntry): boolean {
  if (log.level === "error" || log.level === "warning") return true;
  return KEY_EVENT_MARKERS.some((marker) => log.message.toLowerCase().includes(marker));
}

export function DashboardPage(props: { language: Language; hosts: Host[]; logs: LogEntry[]; onStopAll: () => void }) {
  const lang = props.language;
  const running = props.hosts.flatMap((host) =>
    (host.forwards ?? [])
      .filter((forward) => forward.status === "running")
      .map((forward) => ({ host, forward })),
  );
  const keyEvents = props.logs.filter(isKeyEvent).slice(-8).reverse();

  return (
    <div className="grid gap-5">
      <Card>
        <CardHeader className="flex flex-row items-start justify-between gap-4">
          <div className="space-y-1">
            <CardTitle>{t(lang, "currentConnections")}</CardTitle>
            <CardDescription>{t(lang, "dashboardDesc")}</CardDescription>
          </div>
          {running.length > 0 && (
            <Button variant="secondary" onClick={props.onStopAll}>
              <CircleSlash size={15} />
              {t(lang, "stopAll")}
            </Button>
          )}
        </CardHeader>
        {running.length === 0 ? (
          <p className="rounded-xl border border-dashed border-slate-200 px-4 py-8 text-center text-sm text-slate-400 dark:border-slate-800 dark:text-slate-500">
            {t(lang, "noConnections")}
          </p>
        ) : (
          <div className="grid gap-2">
            {running.map(({ host, forward }) => (
              <div
                key={forward.id}
                className="flex flex-wrap items-center gap-3 rounded-xl border border-emerald-200 bg-emerald-50/60 px-4 py-3 dark:border-emerald-950 dark:bg-emerald-950/20"
              >
                <Server size={16} className="text-emerald-600 dark:text-emerald-400" />
                <div className="min-w-[10rem] flex-1">
                  <div className="text-sm font-semibold text-slate-900 dark:text-slate-100">
                    {host.name} <span className="text-slate-400">/</span> {forward.name}
                  </div>
                  <div className="mt-0.5 text-xs text-slate-500 dark:text-slate-400">
                    <span className="uppercase">{forward.mode}</span>
                    {" · "}
                    {forward.bindDisplay} → {forward.targetDisplay}
                  </div>
                </div>
                <span className="rounded-full bg-emerald-100 px-2.5 py-1 text-xs font-medium text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300">
                  {t(lang, "running")}
                </span>
              </div>
            ))}
          </div>
        )}
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Activity size={16} />
            {t(lang, "keyEvents")}
          </CardTitle>
        </CardHeader>
        {keyEvents.length === 0 ? (
          <p className="px-1 py-4 text-sm text-slate-400 dark:text-slate-500">{t(lang, "noEvents")}</p>
        ) : (
          <div className="grid gap-1.5">
            {keyEvents.map((log, index) => (
              <div
                key={`${log.timestamp}-${index}`}
                className="flex items-center gap-3 rounded-lg px-2 py-1.5 text-sm"
              >
                <span className="whitespace-nowrap text-xs text-slate-400">{log.timestamp}</span>
                <span
                  className={cn(
                    "rounded px-1.5 py-0.5 text-[10px] font-semibold uppercase",
                    log.level === "error"
                      ? "bg-rose-100 text-rose-700 dark:bg-rose-950 dark:text-rose-300"
                      : log.level === "warning"
                        ? "bg-amber-100 text-amber-700 dark:bg-amber-950 dark:text-amber-300"
                        : "bg-slate-100 text-slate-600 dark:bg-slate-800 dark:text-slate-300",
                  )}
                >
                  {log.level}
                </span>
                <span className="truncate text-slate-700 dark:text-slate-200">{log.message}</span>
              </div>
            ))}
          </div>
        )}
      </Card>
    </div>
  );
}
