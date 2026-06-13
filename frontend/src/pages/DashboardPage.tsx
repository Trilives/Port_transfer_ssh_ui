import { Activity, CircleSlash, Globe, Plus, PlugZap, Server } from "lucide-react";
import { Button } from "../components/ui/button";
import { Card, CardDescription, CardHeader, CardTitle } from "../components/ui/card";
import { cn } from "../lib/utils";
import { t } from "../i18n";
import type { Forward, Host, Language, LogEntry } from "../types";

const KEY_EVENT_MARKERS = ["connected", "disconnected", "cleaned up", "exited"];

function isKeyEvent(log: LogEntry): boolean {
  if (log.level === "error" || log.level === "warning") return true;
  return KEY_EVENT_MARKERS.some((marker) => log.message.toLowerCase().includes(marker));
}

export function DashboardPage(props: {
  language: Language;
  hosts: Host[];
  logs: LogEntry[];
  onNew: () => void;
  onStopAll: () => void;
  onDisconnectForward: (host: Host, forward: Forward) => void;
  onOpenForwardWeb: (host: Host, forward: Forward) => void;
}) {
  const lang = props.language;
  const groups = props.hosts
    .map((host) => ({ host, running: (host.forwards ?? []).filter((forward) => forward.status === "running") }))
    .filter((group) => group.running.length > 0);
  const keyEvents = props.logs.filter(isKeyEvent).slice(-8).reverse();

  return (
    <div className="grid gap-5">
      <Card>
        <CardHeader className="flex flex-row items-start justify-between gap-4">
          <div className="space-y-1">
            <CardTitle>{t(lang, "currentConnections")}</CardTitle>
            <CardDescription>{t(lang, "dashboardDesc")}</CardDescription>
          </div>
          <div className="flex shrink-0 gap-2">
            <Button onClick={props.onNew}>
              <Plus size={15} />
              {t(lang, "new")}
            </Button>
            {groups.length > 0 && (
              <Button variant="secondary" onClick={props.onStopAll}>
                <CircleSlash size={15} />
                {t(lang, "stopAll")}
              </Button>
            )}
          </div>
        </CardHeader>

        {groups.length === 0 ? (
          <p className="rounded-xl border border-dashed border-slate-200 px-4 py-8 text-center text-sm text-slate-400 dark:border-slate-800 dark:text-slate-500">
            {t(lang, "noConnections")}
          </p>
        ) : (
          <div className="overflow-hidden rounded-xl border border-slate-200 dark:border-slate-800">
            {groups.map(({ host, running }) => (
              <div key={host.id} className="flex border-b border-slate-200 last:border-b-0 dark:border-slate-800">
                {/* 左栏：窄的主机名，纵向居中，高度随右侧端口数自动撑满 */}
                <div className="flex w-44 shrink-0 items-center gap-2 border-r border-slate-200 bg-slate-50 px-4 py-3 dark:border-slate-800 dark:bg-slate-900/60">
                  <Server size={16} className="shrink-0 text-blue-600 dark:text-blue-300" />
                  <span className="truncate text-sm font-semibold text-slate-900 dark:text-slate-100">{host.name}</span>
                </div>
                {/* 右栏：该主机运行中的端口列表 */}
                <div className="flex-1 divide-y divide-slate-100 dark:divide-slate-800">
                  {running.map((forward) => (
                    <div key={forward.id} className="flex flex-wrap items-center gap-3 px-4 py-2.5">
                      <div className="min-w-[8rem] flex-1">
                        <div className="text-sm font-medium text-slate-900 dark:text-slate-100">{forward.name}</div>
                        <div className="mt-0.5 text-xs text-slate-500 dark:text-slate-400">
                          <span className="uppercase">{forward.mode}</span>
                          {" · "}
                          {forward.bindDisplay} → {forward.targetDisplay}
                        </div>
                      </div>
                      <span className="rounded-full bg-emerald-100 px-2.5 py-1 text-xs font-medium text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300">
                        {t(lang, "running")}
                      </span>
                      <Button variant="ghost" onClick={() => props.onOpenForwardWeb(host, forward)} aria-label={t(lang, "openWeb")} title={t(lang, "openWeb")}>
                        <Globe size={15} />
                      </Button>
                      <Button variant="ghost" onClick={() => props.onDisconnectForward(host, forward)}>
                        <PlugZap size={15} />
                        {t(lang, "disconnect")}
                      </Button>
                    </div>
                  ))}
                </div>
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
              <div key={`${log.timestamp}-${index}`} className="flex items-center gap-3 rounded-lg px-2 py-1.5 text-sm">
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
