import { Plus } from "lucide-react";
import { Button } from "../components/ui/button";
import { Card, CardDescription, CardHeader, CardTitle } from "../components/ui/card";
import { HostCard } from "../components/HostCard";
import { t } from "../i18n";
import type { Forward, Host, Language } from "../types";

export function ConfigPage(props: {
  language: Language;
  hosts: Host[];
  expandedIds: Set<string>;
  onToggle: (hostId: string) => void;
  onNewHost: () => void;
  onEditHost: (host: Host) => void;
  onDeleteHost: (host: Host) => void;
  onSendCommand: (host: Host) => void;
  onOpenTerminal: (host: Host) => void;
  onUploadKey: (host: Host) => void;
  onNewForward: (host: Host) => void;
  onConnectForward: (host: Host, forward: Forward) => void;
  onDisconnectForward: (host: Host, forward: Forward) => void;
  onEditForward: (host: Host, forward: Forward) => void;
  onDeleteForward: (host: Host, forward: Forward) => void;
}) {
  const lang = props.language;
  return (
    <div className="grid gap-5">
      <Card>
        <CardHeader className="flex flex-row items-start justify-between gap-4">
          <div className="space-y-1">
            <CardTitle>{t(lang, "configTitle")}</CardTitle>
            <CardDescription>{t(lang, "configDesc")}</CardDescription>
          </div>
          <Button onClick={props.onNewHost}>
            <Plus size={16} />
            {t(lang, "newHost")}
          </Button>
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
                onOpenTerminal={() => props.onOpenTerminal(host)}
                onUploadKey={() => props.onUploadKey(host)}
                onNewForward={() => props.onNewForward(host)}
                onEditHost={() => props.onEditHost(host)}
                onDeleteHost={() => props.onDeleteHost(host)}
                onConnectForward={(forward) => props.onConnectForward(host, forward)}
                onDisconnectForward={(forward) => props.onDisconnectForward(host, forward)}
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
