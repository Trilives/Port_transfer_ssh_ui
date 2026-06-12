import { useEffect, useRef, useState } from "react";
import { Code2, ChevronDown, ChevronRight, KeyRound, Pencil, Pin, PinOff, Plus, Server, Terminal, Trash2, Zap } from "lucide-react";
import { Button } from "./ui/button";
import { ForwardRow } from "./ForwardRow";
import { cn } from "../lib/utils";
import { t } from "../i18n";
import type { Forward, Host, Language } from "../types";

/** 拆分按钮：左侧点开 PowerShell 终端，右侧小箭头展开菜单（含「通过 VS Code 打开」）。 */
function TerminalSplitButton(props: { language: Language; onOpenTerminal: () => void; onOpenVscode: () => void }) {
  const lang = props.language;
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!open) return;
    const onClick = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    window.addEventListener("mousedown", onClick);
    return () => window.removeEventListener("mousedown", onClick);
  }, [open]);
  return (
    <div ref={ref} className="relative inline-flex">
      <Button variant="secondary" className="rounded-r-none" onClick={props.onOpenTerminal}>
        <Terminal size={15} />
        {t(lang, "openTerminal")}
      </Button>
      <Button
        variant="secondary"
        className="rounded-l-none border-l border-slate-300 px-2 dark:border-slate-600"
        onClick={() => setOpen((v) => !v)}
        aria-label={t(lang, "more")}
        title={t(lang, "more")}
      >
        <ChevronDown size={14} />
      </Button>
      {open && (
        <div className="absolute left-0 top-full z-20 mt-1 w-52 overflow-hidden rounded-xl border border-slate-200 bg-white py-1 shadow-soft dark:border-slate-800 dark:bg-slate-950">
          <button
            onClick={() => {
              setOpen(false);
              props.onOpenVscode();
            }}
            className="flex w-full items-center gap-2 px-4 py-2 text-left text-sm text-slate-700 hover:bg-slate-100 dark:text-slate-200 dark:hover:bg-slate-900"
          >
            <Code2 size={15} />
            {t(lang, "openInVscode")}
          </button>
        </div>
      )}
    </div>
  );
}

export function HostCard(props: {
  language: Language;
  host: Host;
  expanded: boolean;
  onToggle: () => void;
  onSendCommand: () => void;
  onOpenTerminal: () => void;
  onOpenVscode: () => void;
  onUploadKey: () => void;
  onNewForward: () => void;
  onEditHost: () => void;
  onDeleteHost: () => void;
  onTogglePin: () => void;
  onConnectForward: (forward: Forward) => void;
  onDisconnectForward: (forward: Forward) => void;
  onEditForward: (forward: Forward) => void;
  onDeleteForward: (forward: Forward) => void;
}) {
  const lang = props.language;
  const forwards = props.host.forwards ?? [];
  const runningCount = forwards.filter((item) => item.status === "running").length;
  const endpoint = `${props.host.sshUser ? `${props.host.sshUser}@` : ""}${props.host.sshHost || "?"}:${props.host.sshPort || "22"}`;

  return (
    <div className="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-soft dark:border-slate-800 dark:bg-slate-950/70">
      <div className="flex w-full items-center gap-2 pr-4 transition hover:bg-slate-50 dark:hover:bg-slate-900">
        <button onClick={props.onToggle} className="flex flex-1 items-center gap-3 px-5 py-4 text-left">
          {props.expanded ? <ChevronDown size={18} className="text-slate-400" /> : <ChevronRight size={18} className="text-slate-400" />}
          <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-blue-600/10 text-blue-600 dark:bg-blue-500/15 dark:text-blue-300">
            <Server size={18} />
          </div>
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2">
              <span className="truncate text-sm font-semibold text-slate-900 dark:text-slate-100">{props.host.name}</span>
              {props.host.pinned && (
                <span className="inline-flex shrink-0 items-center gap-1 rounded-full bg-blue-100 px-2 py-0.5 text-[11px] font-medium text-blue-700 dark:bg-blue-950 dark:text-blue-300">
                  <Pin size={11} className="fill-current" />
                  {t(lang, "pinned")}
                </span>
              )}
            </div>
            <div className="truncate text-xs text-slate-500 dark:text-slate-400">{endpoint}</div>
          </div>
          <div className="flex items-center gap-2 text-xs text-slate-500 dark:text-slate-400">
            {runningCount > 0 && (
              <span className="rounded-full bg-emerald-100 px-2 py-0.5 font-medium text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300">
                {runningCount} {t(lang, "runningCount")}
              </span>
            )}
            <span>{forwards.length} {t(lang, "forwardsCount")}</span>
          </div>
        </button>
        <Button
          variant="ghost"
          onClick={props.onTogglePin}
          aria-label={t(lang, props.host.pinned ? "unpin" : "pin")}
          title={t(lang, props.host.pinned ? "unpin" : "pin")}
        >
          {props.host.pinned ? (
            <PinOff size={15} className="text-slate-400" />
          ) : (
            <Pin size={15} className="text-blue-600 dark:text-blue-300" />
          )}
        </Button>
        <Button variant="danger" onClick={props.onDeleteHost} aria-label={t(lang, "deleteHost")}>
          <Trash2 size={15} />
        </Button>
      </div>

      {props.expanded && (
        <div className="border-t border-slate-200 px-5 py-4 dark:border-slate-800">
          <div className="mb-4 flex flex-wrap gap-2">
            <Button variant="secondary" onClick={props.onSendCommand}>
              <Zap size={15} />
              {t(lang, "sendCommand")}
            </Button>
            <TerminalSplitButton language={lang} onOpenTerminal={props.onOpenTerminal} onOpenVscode={props.onOpenVscode} />
            <Button variant="secondary" onClick={props.onUploadKey}>
              <KeyRound size={15} />
              {t(lang, "uploadKey")}
            </Button>
            <Button onClick={props.onNewForward}>
              <Plus size={15} />
              {t(lang, "newForward")}
            </Button>
            <Button variant="ghost" className="ml-auto" onClick={props.onEditHost}>
              <Pencil size={15} />
              {t(lang, "editHost")}
            </Button>
          </div>

          {forwards.length === 0 ? (
            <p className={cn("rounded-xl border border-dashed border-slate-200 px-4 py-6 text-center text-sm text-slate-400", "dark:border-slate-800 dark:text-slate-500")}>
              {t(lang, "noForwards")}
            </p>
          ) : (
            <div className="grid gap-2">
              {forwards.map((forward) => (
                <ForwardRow
                  key={forward.id}
                  language={lang}
                  forward={forward}
                  onConnect={() => props.onConnectForward(forward)}
                  onDisconnect={() => props.onDisconnectForward(forward)}
                  onEdit={() => props.onEditForward(forward)}
                  onDelete={() => props.onDeleteForward(forward)}
                />
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
