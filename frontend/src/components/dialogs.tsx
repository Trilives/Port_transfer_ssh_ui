import { Button } from "./ui/button";
import { Card, CardDescription, CardHeader, CardTitle } from "./ui/card";
import { Input } from "./ui/input";
import { t } from "../i18n";
import type { Language } from "../types";

export type CriticalErrorPayload = {
  id: string;
  name: string;
  message: string;
};

export function PasswordDialog(props: {
  language: Language;
  profileName: string;
  password: string;
  setPassword: (value: string) => void;
  onCancel: () => void;
  onSubmit: () => void;
}) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/55 p-6 backdrop-blur-sm">
      <Card className="w-full max-w-md border-blue-200 bg-white dark:border-blue-900 dark:bg-slate-950">
        <CardHeader>
          <CardTitle>{t(props.language, "passwordDialogTitle")}</CardTitle>
          <CardDescription>{t(props.language, "passwordDialogDescription")}: {props.profileName}</CardDescription>
        </CardHeader>
        <div className="grid gap-4">
          <Input
            type="password"
            value={props.password}
            placeholder={t(props.language, "passwordPlaceholder")}
            onChange={(event) => props.setPassword(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") props.onSubmit();
            }}
            autoFocus
          />
          <p className="text-sm leading-6 text-slate-500 dark:text-slate-400">{t(props.language, "passwordOnceNote")}</p>
          <div className="flex justify-end gap-3">
            <Button variant="secondary" onClick={props.onCancel}>{t(props.language, "cancel")}</Button>
            <Button onClick={props.onSubmit} disabled={!props.password}>{t(props.language, "connect")}</Button>
          </div>
        </div>
      </Card>
    </div>
  );
}

export function KeyUploadDialog(props: {
  language: Language;
  profileName: string;
  password: string;
  setPassword: (value: string) => void;
  onCancel: () => void;
  onSubmit: () => void;
}) {
  const isZh = props.language === "zh-CN";
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/55 p-6 backdrop-blur-sm">
      <Card className="w-full max-w-md border-blue-200 bg-white dark:border-blue-900 dark:bg-slate-950">
        <CardHeader>
          <CardTitle>{isZh ? "\u4e0a\u4f20 SSH \u516c\u94a5" : "Upload SSH Public Key"}</CardTitle>
          <CardDescription>
            {isZh
              ? `\u8f93\u5165 ${props.profileName} \u7684 SSH \u5bc6\u7801\uff0c\u7a0b\u5e8f\u4f1a\u628a\u672c\u673a\u516c\u94a5\u5199\u5165\u8fdc\u7aef authorized_keys\u3002`
              : `Enter the SSH password for ${props.profileName}; the app will append your public key to remote authorized_keys.`}
          </CardDescription>
        </CardHeader>
        <div className="grid gap-4">
          <Input
            type="password"
            value={props.password}
            placeholder={isZh ? "SSH \u5bc6\u7801" : "SSH password"}
            onChange={(event) => props.setPassword(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") props.onSubmit();
            }}
            autoFocus
          />
          <p className="text-sm leading-6 text-slate-500 dark:text-slate-400">
            {isZh
              ? "\u5982\u679c\u672a\u6307\u5b9a\u79c1\u94a5\u6587\u4ef6\uff0c\u7a0b\u5e8f\u4f1a\u4f7f\u7528\u6216\u751f\u6210 %USERPROFILE%\\.ssh\\id_ed25519\u3002\u5bc6\u7801\u4e0d\u4f1a\u4fdd\u5b58\u3002"
              : "If no key file is specified, the app uses or creates %USERPROFILE%\\.ssh\\id_ed25519. The password is not saved."}
          </p>
          <div className="flex justify-end gap-3">
            <Button variant="secondary" onClick={props.onCancel}>{t(props.language, "cancel")}</Button>
            <Button onClick={props.onSubmit} disabled={!props.password}>{isZh ? "\u4e0a\u4f20\u5bc6\u94a5" : "Upload Key"}</Button>
          </div>
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
  const isZh = props.language === "zh-CN";
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/55 p-6 backdrop-blur-sm">
      <Card className="w-full max-w-lg border-rose-200 bg-white dark:border-rose-900 dark:bg-slate-950">
        <CardHeader>
          <CardTitle>{isZh ? "\u8fde\u63a5\u51fa\u73b0\u5173\u952e\u9519\u8bef" : "Critical Connection Error"}</CardTitle>
          <CardDescription>
            {isZh
              ? `${props.error.name} \u5df2\u505c\u6b62\u81ea\u52a8\u91cd\u8bd5\u3002\u8bf7\u4fee\u590d\u9519\u8bef\u540e\u624b\u52a8\u91cd\u65b0\u8fde\u63a5\u3002`
              : `${props.error.name} stopped retrying. Fix the error, then reconnect manually.`}
          </CardDescription>
        </CardHeader>
        <pre className="max-h-56 overflow-auto rounded-2xl bg-slate-100 p-4 text-sm leading-6 text-slate-700 dark:bg-slate-900 dark:text-slate-200">
          {props.error.message}
        </pre>
        <div className="mt-4 flex justify-end">
          <Button onClick={props.onClose}>{isZh ? "\u5173\u95ed" : "Close"}</Button>
        </div>
      </Card>
    </div>
  );
}
