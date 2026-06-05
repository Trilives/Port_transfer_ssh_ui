import { cn } from "../lib/utils";
import { t } from "../i18n";
import type { Language, LogEntry } from "../types";

const levelColor: Record<string, string> = {
  error: "text-rose-600 dark:text-rose-400",
  warning: "text-amber-600 dark:text-amber-400",
  info: "text-slate-500 dark:text-slate-400",
  debug: "text-slate-400 dark:text-slate-500",
};

export function LogTable(props: { language: Language; logs: LogEntry[]; maxHeight?: string }) {
  return (
    <div className={cn("overflow-auto rounded-2xl border border-slate-200 dark:border-slate-800", props.maxHeight ?? "max-h-[28rem]")}>
      <table className="w-full text-sm">
        <thead className="sticky top-0 bg-slate-100 text-left text-slate-500 dark:bg-slate-900 dark:text-slate-400">
          <tr>
            <th className="px-4 py-3 font-medium">{t(props.language, "time")}</th>
            <th className="px-4 py-3 font-medium">{t(props.language, "level")}</th>
            <th className="px-4 py-3 font-medium">{t(props.language, "message")}</th>
          </tr>
        </thead>
        <tbody>
          {props.logs.map((log, index) => (
            <tr key={`${log.timestamp}-${index}`} className="border-t border-slate-200 dark:border-slate-800">
              <td className="whitespace-nowrap px-4 py-2 text-slate-500 dark:text-slate-400">{log.timestamp}</td>
              <td className={cn("px-4 py-2 uppercase", levelColor[log.level] ?? "text-slate-500")}>{log.level}</td>
              <td className="px-4 py-2 text-slate-700 dark:text-slate-200">{log.message}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
