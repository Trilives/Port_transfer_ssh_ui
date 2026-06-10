import { type ReactNode } from "react";
import { X } from "lucide-react";
import { Button } from "./ui/button";
import { Card, CardDescription, CardHeader, CardTitle } from "./ui/card";
import { Input } from "./ui/input";
import { Select } from "./ui/select";
import { t } from "../i18n";
import type { CriticalErrorPayload, Forward, Host, Language, TunnelMode } from "../types";

function Modal(props: { children: ReactNode; maxWidth?: string }) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/55 p-6 backdrop-blur-sm">
      <Card
        className={`max-h-[calc(100vh-3rem)] w-full ${props.maxWidth ?? "max-w-xl"} overflow-auto border-blue-200 bg-white dark:border-blue-900 dark:bg-slate-950`}
      >
        {props.children}
      </Card>
    </div>
  );
}

function Field(props: { label: string; value: string; onChange: (value: string) => void; placeholder?: string; hint?: string }) {
  return (
    <label className="grid gap-2 text-sm font-medium text-slate-600 dark:text-slate-300">
      {props.label}
      <Input value={props.value} placeholder={props.placeholder} onChange={(event) => props.onChange(event.target.value)} />
      {props.hint && <span className="text-xs font-normal leading-5 text-slate-400 dark:text-slate-500">{props.hint}</span>}
    </label>
  );
}

function DialogHeader(props: { title: string; description?: string; onClose: () => void }) {
  return (
    <CardHeader className="flex flex-row items-start justify-between gap-4">
      <div className="space-y-1">
        <CardTitle>{props.title}</CardTitle>
        {props.description && <CardDescription>{props.description}</CardDescription>}
      </div>
      <Button variant="ghost" onClick={props.onClose} aria-label="Close">
        <X size={18} />
      </Button>
    </CardHeader>
  );
}

export function HostDialog(props: {
  language: Language;
  draft: Host;
  setDraft: (host: Host) => void;
  onClose: () => void;
  onSave: () => void;
}) {
  const lang = props.language;
  const update = <K extends keyof Host>(key: K, value: Host[K]) => props.setDraft({ ...props.draft, [key]: value });
  return (
    <Modal maxWidth="max-w-2xl">
      <DialogHeader title={t(lang, "hostDialogTitle")} description={t(lang, "hostDialogDesc")} onClose={props.onClose} />
      <div className="grid grid-cols-2 gap-4">
        <Field label={t(lang, "name")} value={props.draft.name} onChange={(v) => update("name", v)} />
        <Field label={t(lang, "sshUser")} value={props.draft.sshUser} onChange={(v) => update("sshUser", v)} />
        <Field label={t(lang, "sshHost")} value={props.draft.sshHost} onChange={(v) => update("sshHost", v)} />
        <Field label={t(lang, "sshPort")} value={props.draft.sshPort} onChange={(v) => update("sshPort", v)} />
        <div className="col-span-2">
          <Field
            label={t(lang, "identityFile")}
            value={props.draft.identityFile}
            onChange={(v) => update("identityFile", v)}
            hint={t(lang, "identityFileHint")}
          />
        </div>
        <div className="col-span-2">
          <Field label={t(lang, "extraOptions")} value={props.draft.extraOptions} onChange={(v) => update("extraOptions", v)} />
        </div>
      </div>
      <div className="mt-5 flex justify-end gap-3">
        <Button variant="secondary" onClick={props.onClose}>{t(lang, "cancel")}</Button>
        <Button onClick={props.onSave}>{t(lang, "save")}</Button>
      </div>
    </Modal>
  );
}

export function ForwardDialog(props: {
  language: Language;
  draft: Forward;
  setDraft: (forward: Forward) => void;
  onClose: () => void;
  onSave: () => void;
}) {
  const lang = props.language;
  const update = <K extends keyof Forward>(key: K, value: Forward[K]) => props.setDraft({ ...props.draft, [key]: value });
  const isDynamic = props.draft.mode === "dynamic";
  return (
    <Modal maxWidth="max-w-2xl">
      <DialogHeader title={t(lang, "forwardDialogTitle")} description={t(lang, "forwardDialogDesc")} onClose={props.onClose} />
      <div className="grid grid-cols-2 gap-4">
        <Field label={t(lang, "name")} value={props.draft.name} onChange={(v) => update("name", v)} />
        <label className="grid gap-2 text-sm font-medium text-slate-600 dark:text-slate-300">
          {t(lang, "mode")}
          <Select value={props.draft.mode} onChange={(e) => update("mode", e.target.value as TunnelMode)}>
            <option value="local">local</option>
            <option value="remote">remote</option>
            <option value="dynamic">dynamic</option>
          </Select>
        </label>
        <Field label={t(lang, "bindHost")} value={props.draft.bindHost} onChange={(v) => update("bindHost", v)} />
        <Field label={t(lang, "bindPort")} value={props.draft.bindPort} onChange={(v) => update("bindPort", v)} />
        {!isDynamic && (
          <>
            <Field label={t(lang, "targetHost")} value={props.draft.targetHost} onChange={(v) => update("targetHost", v)} />
            <Field label={t(lang, "targetPort")} value={props.draft.targetPort} onChange={(v) => update("targetPort", v)} />
          </>
        )}
        <label className="col-span-2 flex items-center gap-3 rounded-2xl bg-slate-50 p-4 text-sm font-medium dark:bg-slate-900">
          <input
            type="checkbox"
            checked={props.draft.keepConnected}
            onChange={(e) => update("keepConnected", e.target.checked)}
            className="h-4 w-4 rounded border-slate-300"
          />
          {t(lang, "keepConnected")}
        </label>
      </div>
      <div className="mt-5 flex justify-end gap-3">
        <Button variant="secondary" onClick={props.onClose}>{t(lang, "cancel")}</Button>
        <Button onClick={props.onSave}>{t(lang, "save")}</Button>
      </div>
    </Modal>
  );
}

export function SendCommandDialog(props: {
  language: Language;
  hostName: string;
  command: string;
  setCommand: (value: string) => void;
  output: string;
  busy: boolean;
  onClose: () => void;
  onRun: () => void;
  onRunWithPassword: () => void;
}) {
  const lang = props.language;
  const disabled = props.busy || !props.command.trim();
  return (
    <Modal maxWidth="max-w-2xl">
      <DialogHeader
        title={`${t(lang, "sendCommandTitle")} — ${props.hostName}`}
        description={t(lang, "sendCommandDesc")}
        onClose={props.onClose}
      />
      <div className="grid gap-4">
        <Input
          value={props.command}
          placeholder={t(lang, "commandPlaceholder")}
          onChange={(e) => props.setCommand(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !disabled) props.onRun();
          }}
          autoFocus
        />
        {(props.output || props.busy) && (
          <div>
            <div className="mb-2 text-xs font-medium uppercase tracking-wide text-slate-400">{t(lang, "output")}</div>
            <pre className="max-h-64 overflow-auto rounded-2xl bg-slate-100 p-4 text-xs leading-6 text-slate-700 dark:bg-slate-900 dark:text-slate-200">
              {props.busy ? t(lang, "running2") : props.output}
            </pre>
          </div>
        )}
        <div className="flex justify-end gap-3">
          <Button variant="secondary" onClick={props.onClose}>{t(lang, "close")}</Button>
          <Button variant="secondary" onClick={props.onRunWithPassword} disabled={disabled}>{t(lang, "sendWithPassword")}</Button>
          <Button onClick={props.onRun} disabled={disabled}>{t(lang, "run")}</Button>
        </div>
      </div>
    </Modal>
  );
}

/** 通用一次性密码输入弹窗（层级高于其他弹窗）。 */
export function InputPasswordDialog(props: {
  language: Language;
  title: string;
  description?: string;
  submitLabel: string;
  password: string;
  setPassword: (value: string) => void;
  onCancel: () => void;
  onSubmit: () => void;
}) {
  const lang = props.language;
  return (
    <div className="fixed inset-0 z-[60] flex items-center justify-center bg-slate-950/55 p-6 backdrop-blur-sm">
      <Card className="w-full max-w-md border-blue-200 bg-white dark:border-blue-900 dark:bg-slate-950">
        <CardHeader>
          <CardTitle>{props.title}</CardTitle>
          {props.description && <CardDescription>{props.description}</CardDescription>}
        </CardHeader>
        <div className="grid gap-4">
          <Input
            type="password"
            value={props.password}
            placeholder={t(lang, "passwordPlaceholder")}
            onChange={(e) => props.setPassword(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") props.onSubmit();
            }}
            autoFocus
          />
          <div className="flex justify-end gap-3">
            <Button variant="secondary" onClick={props.onCancel}>{t(lang, "cancel")}</Button>
            <Button onClick={props.onSubmit} disabled={!props.password}>{props.submitLabel}</Button>
          </div>
        </div>
      </Card>
    </div>
  );
}

export function PasswordDialog(props: {
  language: Language;
  targetName: string;
  password: string;
  setPassword: (value: string) => void;
  onCancel: () => void;
  onSubmit: () => void;
}) {
  const lang = props.language;
  return (
    <Modal maxWidth="max-w-md">
      <CardHeader>
        <CardTitle>{t(lang, "passwordDialogTitle")}</CardTitle>
        <CardDescription>{t(lang, "passwordDialogDescription")}：{props.targetName}</CardDescription>
      </CardHeader>
      <div className="grid gap-4">
        <Input
          type="password"
          value={props.password}
          placeholder={t(lang, "passwordPlaceholder")}
          onChange={(e) => props.setPassword(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") props.onSubmit();
          }}
          autoFocus
        />
        <p className="text-sm leading-6 text-slate-500 dark:text-slate-400">{t(lang, "passwordOnceNote")}</p>
        <div className="flex justify-end gap-3">
          <Button variant="secondary" onClick={props.onCancel}>{t(lang, "cancel")}</Button>
          <Button onClick={props.onSubmit} disabled={!props.password}>{t(lang, "connect")}</Button>
        </div>
      </div>
    </Modal>
  );
}

export function KeyUploadDialog(props: {
  language: Language;
  hostName: string;
  password: string;
  setPassword: (value: string) => void;
  onCancel: () => void;
  onSubmit: () => void;
}) {
  const lang = props.language;
  return (
    <Modal maxWidth="max-w-md">
      <CardHeader>
        <CardTitle>{t(lang, "keyUploadTitle")}</CardTitle>
        <CardDescription>{props.hostName} — {t(lang, "keyUploadDesc")}</CardDescription>
      </CardHeader>
      <div className="grid gap-4">
        <Input
          type="password"
          value={props.password}
          placeholder={t(lang, "passwordPlaceholder")}
          onChange={(e) => props.setPassword(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") props.onSubmit();
          }}
          autoFocus
        />
        <p className="text-sm leading-6 text-slate-500 dark:text-slate-400">{t(lang, "keyUploadNote")}</p>
        <div className="flex justify-end gap-3">
          <Button variant="secondary" onClick={props.onCancel}>{t(lang, "cancel")}</Button>
          <Button onClick={props.onSubmit} disabled={!props.password}>{t(lang, "uploadKey")}</Button>
        </div>
      </div>
    </Modal>
  );
}

export function HostKeyChangedDialog(props: {
  language: Language;
  hostName: string;
  fingerprint: string;
  fetching: boolean;
  onCancel: () => void;
  onTrust: () => void;
}) {
  const lang = props.language;
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/55 p-6 backdrop-blur-sm">
      <Card className="w-full max-w-lg border-amber-300 bg-white dark:border-amber-800 dark:bg-slate-950">
        <CardHeader>
          <CardTitle>⚠️ {t(lang, "hostKeyChangedTitle")}</CardTitle>
          <CardDescription>{props.hostName} — {t(lang, "hostKeyChangedWarn")}</CardDescription>
        </CardHeader>
        <div className="grid gap-3">
          <p className="text-sm font-medium text-slate-600 dark:text-slate-300">{t(lang, "hostKeyFingerprintLabel")}</p>
          <pre className="max-h-40 overflow-auto rounded-2xl bg-slate-100 p-4 text-xs leading-6 text-slate-700 dark:bg-slate-900 dark:text-slate-200">
            {props.fetching ? t(lang, "hostKeyFetching") : props.fingerprint || t(lang, "hostKeyUnavailable")}
          </pre>
          <div className="flex justify-end gap-3">
            <Button onClick={props.onCancel}>{t(lang, "cancel")}</Button>
            <Button variant="danger" onClick={props.onTrust}>{t(lang, "hostKeyTrust")}</Button>
          </div>
        </div>
      </Card>
    </div>
  );
}

export function ConfirmDialog(props: {
  language: Language;
  title: string;
  description: string;
  confirmLabel: string;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  const lang = props.language;
  return (
    <div className="fixed inset-0 z-[60] flex items-center justify-center bg-slate-950/55 p-6 backdrop-blur-sm">
      <Card className="w-full max-w-md border-rose-200 bg-white dark:border-rose-900 dark:bg-slate-950">
        <CardHeader>
          <CardTitle>{props.title}</CardTitle>
          <CardDescription>{props.description}</CardDescription>
        </CardHeader>
        <div className="mt-2 flex justify-end gap-3">
          <Button variant="secondary" onClick={props.onCancel}>{t(lang, "cancel")}</Button>
          <Button variant="danger" onClick={props.onConfirm}>{props.confirmLabel}</Button>
        </div>
      </Card>
    </div>
  );
}

export function SshMissingDialog(props: {
  language: Language;
  onCancel: () => void;
  onInstall: () => void;
}) {
  const lang = props.language;
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/55 p-6 backdrop-blur-sm">
      <Card className="w-full max-w-md border-amber-300 bg-white dark:border-amber-800 dark:bg-slate-950">
        <CardHeader>
          <CardTitle>⚠️ {t(lang, "sshMissingTitle")}</CardTitle>
          <CardDescription>{t(lang, "sshMissingDesc")}</CardDescription>
        </CardHeader>
        <div className="mt-2 flex justify-end gap-3">
          <Button variant="secondary" onClick={props.onCancel}>{t(lang, "cancel")}</Button>
          <Button onClick={props.onInstall}>{t(lang, "install")}</Button>
        </div>
      </Card>
    </div>
  );
}

export function ConnectionErrorDialog(props: {
  language: Language;
  message: string;
  onClose: () => void;
}) {
  const lang = props.language;
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/55 p-6 backdrop-blur-sm">
      <Card className="w-full max-w-lg border-rose-200 bg-white dark:border-rose-900 dark:bg-slate-950">
        <CardHeader>
          <CardTitle>{t(lang, "connectFailedTitle")}</CardTitle>
          <CardDescription>{t(lang, "connectFailedDesc")}</CardDescription>
        </CardHeader>
        <pre className="max-h-56 overflow-auto whitespace-pre-wrap rounded-2xl bg-slate-100 p-4 text-sm leading-6 text-slate-700 dark:bg-slate-900 dark:text-slate-200">
          {props.message}
        </pre>
        <div className="mt-4 flex justify-end">
          <Button onClick={props.onClose}>{t(lang, "close")}</Button>
        </div>
      </Card>
    </div>
  );
}

export function CriticalErrorDialog(props: {
  language: Language;
  error: CriticalErrorPayload;
  onClose: () => void;
}) {
  const lang = props.language;
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/55 p-6 backdrop-blur-sm">
      <Card className="w-full max-w-lg border-rose-200 bg-white dark:border-rose-900 dark:bg-slate-950">
        <CardHeader>
          <CardTitle>{t(lang, "criticalTitle")}</CardTitle>
          <CardDescription>{props.error.name} — {t(lang, "criticalDesc")}</CardDescription>
        </CardHeader>
        <pre className="max-h-56 overflow-auto rounded-2xl bg-slate-100 p-4 text-sm leading-6 text-slate-700 dark:bg-slate-900 dark:text-slate-200">
          {props.error.message}
        </pre>
        <div className="mt-4 flex justify-end">
          <Button onClick={props.onClose}>{t(lang, "close")}</Button>
        </div>
      </Card>
    </div>
  );
}
