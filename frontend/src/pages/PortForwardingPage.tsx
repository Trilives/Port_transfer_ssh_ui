import { useEffect, useRef, useState } from "react";
import { ChevronDown, Download, Plus, Upload } from "lucide-react";
import { Button } from "../components/ui/button";
import { Card, CardDescription, CardHeader, CardTitle } from "../components/ui/card";
import { HostCard } from "../components/HostCard";
import { t } from "../i18n";
import type { Forward, Host, Language } from "../types";

/** Small dropdown menu: click the button to expand, click a menu item or outside to close. */
function Dropdown(props: {
  label: string;
  icon: typeof Download;
  items: { label: string; onClick: () => void }[];
}) {
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
  const Icon = props.icon;
  return (
    <div ref={ref} className="relative">
      <Button variant="secondary" onClick={() => setOpen((v) => !v)}>
        <Icon size={16} />
        {props.label}
        <ChevronDown size={14} />
      </Button>
      {open && (
        <div className="absolute right-0 z-20 mt-2 w-56 overflow-hidden rounded-xl border border-slate-200 bg-white py-1 shadow-soft dark:border-slate-800 dark:bg-slate-950">
          {props.items.map((item) => (
            <button
              key={item.label}
              onClick={() => {
                setOpen(false);
                item.onClick();
              }}
              className="block w-full px-4 py-2 text-left text-sm text-slate-700 hover:bg-slate-100 dark:text-slate-200 dark:hover:bg-slate-900"
            >
              {item.label}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

export function PortForwardingPage(props: {
  language: Language;
  hosts: Host[];
  expandedIds: Set<string>;
  onToggle: (hostId: string) => void;
  onNewHost: () => void;
  onEditHost: (host: Host) => void;
  onDeleteHost: (host: Host) => void;
  onTogglePin: (host: Host) => void;
  onSendCommand: (host: Host) => void;
  onUploadKey: (host: Host) => void;
  onNewForward: (host: Host) => void;
  onConnectForward: (host: Host, forward: Forward) => void;
  onDisconnectForward: (host: Host, forward: Forward) => void;
  onOpenForwardWeb: (host: Host, forward: Forward) => void;
  onEditForward: (host: Host, forward: Forward) => void;
  onDeleteForward: (host: Host, forward: Forward) => void;
  onImportFromFile: () => void;
  onImportFromConfig: () => void;
  onExportToFile: () => void;
  onExportToConfig: () => void;
}) {
  const lang = props.language;
  return (
    <div className="grid gap-5">
      <Card>
        <CardHeader className="flex flex-wrap items-start justify-between gap-4">
          <div className="min-w-[16rem] flex-1 space-y-1">
            <CardTitle>{t(lang, "configTitle")}</CardTitle>
            <CardDescription>{t(lang, "configDesc")}</CardDescription>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Dropdown
              label={t(lang, "import")}
              icon={Download}
              items={[
                { label: t(lang, "importFromFile"), onClick: props.onImportFromFile },
                { label: t(lang, "importFromConfig"), onClick: props.onImportFromConfig },
              ]}
            />
            <Dropdown
              label={t(lang, "export")}
              icon={Upload}
              items={[
                { label: t(lang, "exportToFile"), onClick: props.onExportToFile },
                { label: t(lang, "exportToConfig"), onClick: props.onExportToConfig },
              ]}
            />
            <Button onClick={props.onNewHost}>
              <Plus size={16} />
              {t(lang, "newHost")}
            </Button>
          </div>
        </CardHeader>
        {props.hosts.length === 0 ? (
          <p className="rounded-xl border border-dashed border-slate-200 px-4 py-10 text-center text-sm text-slate-400 dark:border-slate-800 dark:text-slate-500">
            {t(lang, "noHosts")}
          </p>
        ) : (
          <div className="grid gap-3">
            {props.hosts.map((host) => (
              <HostCard
                key={host.id}
                language={lang}
                host={host}
                expanded={props.expandedIds.has(host.id)}
                onToggle={() => props.onToggle(host.id)}
                onSendCommand={() => props.onSendCommand(host)}
                onUploadKey={() => props.onUploadKey(host)}
                onNewForward={() => props.onNewForward(host)}
                onEditHost={() => props.onEditHost(host)}
                onDeleteHost={() => props.onDeleteHost(host)}
                onTogglePin={() => props.onTogglePin(host)}
                onConnectForward={(forward) => props.onConnectForward(host, forward)}
                onDisconnectForward={(forward) => props.onDisconnectForward(host, forward)}
                onOpenForwardWeb={(forward) => props.onOpenForwardWeb(host, forward)}
                onEditForward={(forward) => props.onEditForward(host, forward)}
                onDeleteForward={(forward) => props.onDeleteForward(host, forward)}
              />
            ))}
          </div>
        )}
      </Card>
    </div>
  );
}
