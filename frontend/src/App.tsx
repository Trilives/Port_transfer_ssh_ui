import { useEffect, useMemo, useState, type ReactNode } from "react";
import { Activity, BookOpen, CheckCircle2, History, KeyRound, Plug, Settings, SlidersHorizontal, Terminal, Trash2, X } from "lucide-react";
import { api } from "./api";
import { Button } from "./components/ui/button";
import { Card, CardDescription, CardHeader, CardTitle } from "./components/ui/card";
import { Input } from "./components/ui/input";
import { Select } from "./components/ui/select";
import { languageLabels, t } from "./i18n";
import { cn } from "./lib/utils";
import type { AppSettings, Language, LogEntry, LogLevel, ThemeName, TunnelMode, TunnelProfile } from "./types";

type Page = "main" | "settings" | "history" | "guide";

const defaultProfile = (): TunnelProfile => ({
  id: crypto.randomUUID(),
  name: "new tunnel",
  mode: "local",
  sshHost: "",
  sshPort: "22",
  sshUser: "",
  identityFile: "",
  bindHost: "127.0.0.1",
  localPort: "",
  remoteHost: "127.0.0.1",
  remotePort: "",
  extraOptions: "",
  keepConnected: true,
});

const defaultSettings: AppSettings = {
  theme: "dark",
  language: "zh-CN",
  logLevel: "info",
};

export function App() {
  const [page, setPage] = useState<Page>("history");
  const [profiles, setProfiles] = useState<TunnelProfile[]>([]);
  const [selectedId, setSelectedId] = useState<string>("");
  const [draft, setDraft] = useState<TunnelProfile>(defaultProfile());
  const [settings, setSettings] = useState<AppSettings>(defaultSettings);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [error, setError] = useState("");
  const [passwordTarget, setPasswordTarget] = useState<TunnelProfile | null>(null);
  const [passwordValue, setPasswordValue] = useState("");
  const [profileDialogOpen, setProfileDialogOpen] = useState(false);

  const selected = useMemo(() => profiles.find((item) => item.id === selectedId), [profiles, selectedId]);
  const language = settings.language;

  useEffect(() => {
    document.documentElement.classList.toggle("dark", settings.theme === "dark");
  }, [settings.theme]);

  useEffect(() => {
    void refreshAll();
    const timer = window.setInterval(() => {
      void refreshProfiles();
      void refreshLogs(settings.logLevel);
    }, 1500);
    return () => window.clearInterval(timer);
  }, [settings.logLevel]);

  async function refreshAll() {
    try {
      const [nextSettings, nextProfiles] = await Promise.all([api.getSettings(), api.listProfiles()]);
      setSettings(nextSettings);
      setProfiles(nextProfiles);
      await refreshLogs(nextSettings.logLevel);
    } catch (err) {
      setError(String(err));
    }
  }

  async function refreshProfiles() {
    const nextProfiles = await api.listProfiles();
    setProfiles(nextProfiles);
  }

  async function refreshLogs(level: LogLevel) {
    setLogs(await api.listLogs(level));
  }

  function selectProfile(profile: TunnelProfile) {
    if (selectedId === profile.id) {
      setSelectedId("");
      setDraft(defaultProfile());
      return;
    }
    setSelectedId(profile.id);
    setDraft(profile);
  }

  async function saveDraft() {
    try {
      const saved = await api.saveProfile(draft);
      setSelectedId(saved.id);
      setDraft(saved);
      await refreshProfiles();
      setError("");
      return saved;
    } catch (err) {
      setError(String(err));
      return undefined;
    }
  }

  async function connect(id = selectedId) {
    if (!id) return;
    try {
      await api.connectProfile(id);
      await refreshProfiles();
      setPage("main");
    } catch (err) {
      setError(String(err));
    }
  }

  function requestPasswordConnect(id = selectedId) {
    const profile = profiles.find((item) => item.id === id);
    if (profile) {
      setPasswordTarget(profile);
      setPasswordValue("");
    }
  }

  async function connectWithPassword() {
    if (!passwordTarget || !passwordValue) return;
    try {
      await api.connectProfileWithPassword(passwordTarget.id, passwordValue);
      setPasswordTarget(null);
      setPasswordValue("");
      setProfileDialogOpen(false);
      await refreshProfiles();
      setPage("main");
    } catch (err) {
      setError(String(err));
    }
  }

  async function disconnect(id = selectedId) {
    if (!id) return;
    await api.disconnectProfile(id);
    await refreshProfiles();
  }

  async function removeProfile(id = selectedId) {
    if (!id) return;
    try {
      await api.deleteProfile(id);
      const remaining = profiles.filter((item) => item.id !== id);
      setProfiles(remaining);
      setSelectedId(remaining[0]?.id ?? "");
      setDraft(remaining[0] ?? defaultProfile());
    } catch (err) {
      setError(String(err));
    }
  }

  async function updateSettings(next: Partial<AppSettings>) {
    const merged = { ...settings, ...next };
    setSettings(await api.saveSettings(merged));
  }

  function openNewProfileDialog() {
    const profile = defaultProfile();
    setDraft(profile);
    setSelectedId("");
    setProfileDialogOpen(true);
  }

  const nav = [
    { id: "main" as const, label: t(language, "main"), icon: Activity },
    { id: "history" as const, label: t(language, "history"), icon: History },
    { id: "settings" as const, label: t(language, "settings"), icon: Settings },
    { id: "guide" as const, label: t(language, "guide"), icon: BookOpen },
  ];

  return (
    <main className="min-h-screen bg-slate-50 text-slate-950 transition duration-300 dark:bg-[#090d18] dark:text-slate-50">
      <div className="flex min-h-screen">
        <aside className="w-72 border-r border-slate-200/80 bg-white/80 p-5 backdrop-blur dark:border-slate-800 dark:bg-slate-950/60">
          <div className="mb-8">
            <div className="flex h-11 w-11 items-center justify-center rounded-2xl bg-blue-600 text-white shadow-soft">
              <Terminal size={22} />
            </div>
            <h1 className="mt-4 text-2xl font-semibold tracking-normal">{t(language, "title")}</h1>
            <p className="mt-2 text-sm leading-6 text-slate-500 dark:text-slate-400">{t(language, "subtitle")}</p>
          </div>
          <nav className="space-y-2">
            {nav.map((item) => (
              <button
                key={item.id}
                onClick={() => setPage(item.id)}
                className={cn(
                  "flex w-full items-center gap-3 rounded-2xl px-4 py-3 text-left text-sm font-medium transition duration-200",
                  page === item.id
                    ? "bg-blue-600 text-white shadow-soft"
                    : "text-slate-600 hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-900",
                )}
              >
                <item.icon size={18} />
                {item.label}
              </button>
            ))}
          </nav>
        </aside>

        <section className="flex-1 p-6">
          {error && (
            <div className="mb-4 rounded-2xl border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-700 dark:border-rose-950 dark:bg-rose-950/40 dark:text-rose-200">
              {error}
            </div>
          )}
          <div className="animate-[fadeIn_220ms_ease-out]">
            {page === "main" && (
              <MainPage
                language={language}
                profiles={profiles}
                logs={logs}
                selectedId={selectedId}
                onSelect={selectProfile}
                onNew={openNewProfileDialog}
                onConnect={() => connect()}
                onPasswordConnect={() => requestPasswordConnect()}
                onDisconnect={() => disconnect()}
                onDisconnectAll={async () => {
                  await api.disconnectAll();
                  await refreshProfiles();
                }}
              />
            )}
            {page === "settings" && <SettingsPage settings={settings} setSettings={updateSettings} />}
            {page === "history" && (
              <HistoryPage
                language={language}
                profiles={profiles}
                selectedId={selectedId}
                onSelect={selectProfile}
                onNew={openNewProfileDialog}
                onEdit={(profile) => {
                  setSelectedId(profile.id);
                  setDraft(profile);
                  setProfileDialogOpen(true);
                }}
                onConnect={connect}
                onPasswordConnect={requestPasswordConnect}
                onDisconnect={disconnect}
                onDelete={removeProfile}
              />
            )}
            {page === "guide" && <CleanGuidePage language={language} />}
          </div>
        </section>
      </div>
      {profileDialogOpen && (
        <ProfileDialog
          language={language}
          draft={draft}
          setDraft={setDraft}
          onClose={() => setProfileDialogOpen(false)}
          onNew={openNewProfileDialog}
          onSave={saveDraft}
          onConnect={async () => {
            const saved = await saveDraft();
            if (saved) {
              setProfileDialogOpen(false);
              await connect(saved.id);
            }
          }}
          onPasswordConnect={async () => {
            const saved = await saveDraft();
            if (saved) {
              setProfileDialogOpen(false);
              setPasswordTarget(saved);
              setPasswordValue("");
            }
          }}
          onDisconnect={() => disconnect(draft.id)}
        />
      )}
      {passwordTarget && (
        <PasswordDialog
          language={language}
          profileName={passwordTarget.name}
          password={passwordValue}
          setPassword={setPasswordValue}
          onCancel={() => {
            setPasswordTarget(null);
            setPasswordValue("");
          }}
          onSubmit={connectWithPassword}
        />
      )}
    </main>
  );
}

function MainPage(props: {
  language: Language;
  profiles: TunnelProfile[];
  logs: LogEntry[];
  selectedId: string;
  onSelect: (profile: TunnelProfile) => void;
  onNew: () => void;
  onConnect: () => void;
  onPasswordConnect: () => void;
  onDisconnect: () => void;
  onDisconnectAll: () => void;
}) {
  return (
    <div className="grid gap-5">
      <div className="grid grid-cols-[1fr_280px] gap-5">
        <Card>
          <CardHeader>
            <CardTitle>{t(props.language, "current")}</CardTitle>
            <CardDescription>{t(props.language, "currentGuide")}</CardDescription>
          </CardHeader>
          <ProfileTable language={props.language} profiles={props.profiles} selectedId={props.selectedId} onSelect={props.onSelect} compact />
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>{t(props.language, "quick")}</CardTitle>
          </CardHeader>
          <div className="grid gap-3">
            <Button variant="secondary" onClick={props.onNew}>
              <SlidersHorizontal size={16} />
              {t(props.language, "new")}
            </Button>
            <Button onClick={props.onConnect}>
              <Plug size={16} />
              {t(props.language, "connectSelected")}
            </Button>
            <Button variant="secondary" onClick={props.onPasswordConnect}>
              <KeyRound size={16} />
              {t(props.language, "connectWithPassword")}
            </Button>
            <Button variant="secondary" onClick={props.onDisconnect}>
              {t(props.language, "disconnectSelected")}
            </Button>
            <Button variant="secondary" onClick={props.onDisconnectAll}>
              {t(props.language, "stopAll")}
            </Button>
          </div>
        </Card>
      </div>
      <Card>
        <CardHeader>
          <CardTitle>{t(props.language, "logs")}</CardTitle>
          <CardDescription>{t(props.language, "logsGuide")}</CardDescription>
        </CardHeader>
        <LogTable language={props.language} logs={props.logs} />
      </Card>
    </div>
  );
}

function ProfileDialog(props: {
  language: Language;
  draft: TunnelProfile;
  setDraft: (profile: TunnelProfile) => void;
  onClose: () => void;
  onNew: () => void;
  onSave: () => void;
  onConnect: () => void;
  onPasswordConnect: () => void;
  onDisconnect: () => void;
}) {
  const update = <K extends keyof TunnelProfile>(key: K, value: TunnelProfile[K]) => props.setDraft({ ...props.draft, [key]: value });
  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center bg-slate-950/55 p-6 backdrop-blur-sm">
      <Card className="max-h-[calc(100vh-3rem)] w-full max-w-5xl overflow-auto border-blue-200 bg-white dark:border-blue-900 dark:bg-slate-950">
        <CardHeader className="flex flex-row items-start justify-between gap-4">
          <div>
            <CardTitle>{t(props.language, "profile")}</CardTitle>
            <CardDescription>{t(props.language, "profileGuide")}</CardDescription>
          </div>
          <Button variant="ghost" onClick={props.onClose} aria-label="Close">
            <X size={18} />
          </Button>
        </CardHeader>
        <div className="grid grid-cols-2 gap-4">
          <Field label={t(props.language, "name")} value={props.draft.name} onChange={(value) => update("name", value)} />
          <label className="grid gap-2 text-sm font-medium text-slate-600 dark:text-slate-300">
            {t(props.language, "mode")}
            <Select value={props.draft.mode} onChange={(event) => update("mode", event.target.value as TunnelMode)}>
              <option value="local">local</option>
              <option value="remote">remote</option>
              <option value="dynamic">dynamic</option>
            </Select>
          </label>
          <Field label={t(props.language, "sshHost")} value={props.draft.sshHost} onChange={(value) => update("sshHost", value)} />
          <Field label={t(props.language, "sshPort")} value={props.draft.sshPort} onChange={(value) => update("sshPort", value)} />
          <Field label={t(props.language, "sshUser")} value={props.draft.sshUser} onChange={(value) => update("sshUser", value)} />
          <Field label={t(props.language, "identityFile")} value={props.draft.identityFile} onChange={(value) => update("identityFile", value)} />
          <Field label={t(props.language, "bindHost")} value={props.draft.bindHost} onChange={(value) => update("bindHost", value)} />
          <Field label={t(props.language, "localPort")} value={props.draft.localPort} onChange={(value) => update("localPort", value)} />
          <Field label={t(props.language, "remoteHost")} value={props.draft.remoteHost} onChange={(value) => update("remoteHost", value)} />
          <Field label={t(props.language, "remotePort")} value={props.draft.remotePort} onChange={(value) => update("remotePort", value)} />
          <div className="col-span-2">
            <Field label={t(props.language, "extraOptions")} value={props.draft.extraOptions} onChange={(value) => update("extraOptions", value)} />
          </div>
          <label className="col-span-2 flex items-center gap-3 rounded-2xl bg-slate-50 p-4 text-sm font-medium dark:bg-slate-900">
            <input
              type="checkbox"
              checked={props.draft.keepConnected}
              onChange={(event) => update("keepConnected", event.target.checked)}
              className="h-4 w-4 rounded border-slate-300"
            />
            {t(props.language, "keepConnected")}
          </label>
        </div>
        <div className="mt-5 flex flex-wrap gap-3">
          <Button variant="secondary" onClick={props.onNew}>{t(props.language, "new")}</Button>
          <Button variant="secondary" onClick={props.onSave}>{t(props.language, "save")}</Button>
          <Button onClick={props.onConnect}>{t(props.language, "connect")}</Button>
          <Button variant="secondary" onClick={props.onPasswordConnect}>{t(props.language, "connectWithPassword")}</Button>
          <Button variant="secondary" onClick={props.onDisconnect}>{t(props.language, "disconnect")}</Button>
        </div>
      </Card>
    </div>
  );
}

function SettingsPage(props: { settings: AppSettings; setSettings: (settings: Partial<AppSettings>) => void }) {
  const language = props.settings.language;
  return (
    <Card className="max-w-3xl">
      <CardHeader>
        <CardTitle>{t(language, "settings")}</CardTitle>
        <CardDescription>Theme, language, and log level are saved locally by the Rust backend.</CardDescription>
      </CardHeader>
      <div className="grid gap-4">
        <SettingSelect label={t(language, "theme")} value={props.settings.theme} onChange={(value) => props.setSettings({ theme: value as ThemeName })}>
          <option value="dark">dark</option>
          <option value="light">light</option>
        </SettingSelect>
        <SettingSelect label={t(language, "language")} value={props.settings.language} onChange={(value) => props.setSettings({ language: value as Language })}>
          {Object.entries(languageLabels).map(([value, label]) => <option key={value} value={value}>{label}</option>)}
        </SettingSelect>
        <SettingSelect label={t(language, "logLevel")} value={props.settings.logLevel} onChange={(value) => props.setSettings({ logLevel: value as LogLevel })}>
          <option value="debug">debug</option>
          <option value="info">info</option>
          <option value="warning">warning</option>
          <option value="error">error</option>
        </SettingSelect>
      </div>
    </Card>
  );
}

function HistoryPage(props: {
  language: Language;
  profiles: TunnelProfile[];
  selectedId: string;
  onSelect: (profile: TunnelProfile) => void;
  onNew: () => void;
  onEdit: (profile: TunnelProfile) => void;
  onConnect: (id: string) => void;
  onPasswordConnect: (id: string) => void;
  onDisconnect: (id: string) => void;
  onDelete: (id: string) => void;
}) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-start justify-between gap-4">
        <div>
          <CardTitle>{t(props.language, "history")}</CardTitle>
          <CardDescription>
            {props.language === "zh-CN"
              ? "\u7a0b\u5e8f\u9ed8\u8ba4\u6253\u5f00\u8fd9\u91cc\u3002\u70b9\u51fb\u65b0\u5efa\u6216\u914d\u7f6e\u4f1a\u5f39\u51fa\u53c2\u6570\u7a97\u53e3\uff0c\u70b9\u51fb\u8fde\u63a5\u540e\u56de\u5230\u4e3b\u9875\u9762\u3002"
              : "The app opens here by default. Click New or Configure to edit parameters in a popup, then connect to return to Main."}
          </CardDescription>
        </div>
        <Button onClick={props.onNew}>
          <SlidersHorizontal size={16} />
          {t(props.language, "new")}
        </Button>
      </CardHeader>
      <ProfileTable
        language={props.language}
        profiles={props.profiles}
        selectedId={props.selectedId}
        onSelect={props.onSelect}
        actions={(profile) => (
          <div className="flex flex-wrap gap-2">
            <Button variant="ghost" onClick={() => props.onEdit(profile)}>{t(props.language, "config")}</Button>
            <Button variant="ghost" onClick={() => props.onConnect(profile.id)}>{t(props.language, "connect")}</Button>
            <Button variant="ghost" onClick={() => props.onPasswordConnect(profile.id)}>{t(props.language, "connectWithPassword")}</Button>
            <Button variant="ghost" onClick={() => props.onDisconnect(profile.id)}>{t(props.language, "disconnect")}</Button>
            <Button variant="danger" onClick={() => props.onDelete(profile.id)}>
              <Trash2 size={15} />
            </Button>
          </div>
        )}
      />
    </Card>
  );
}

function PasswordDialog(props: {
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
          <CardDescription>{t(props.language, "passwordDialogDescription")}：{props.profileName}</CardDescription>
        </CardHeader>
        <div className="grid gap-4">
          <Input
            type="password"
            value={props.password}
            placeholder={t(props.language, "passwordPlaceholder")}
            onChange={(event) => props.setPassword(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                props.onSubmit();
              }
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

function CleanGuidePage(props: { language: Language }) {
  const isZh = props.language === "zh-CN";
  const quickSteps = isZh
    ? [
        ["1", "新建连接", "进入连接管理或主页面，点击新建，打开配置弹窗。"],
        ["2", "填写参数", "选择 Local/Remote/Dynamic，填 SSH 主机、用户、监听端口和目标地址。"],
        ["3", "开始连接", "点击连接后会保存配置、关闭弹窗，并回到主页面查看状态和日志。"],
      ]
    : [
        ["1", "Create", "Open Connection Management or Main, then click New to open the profile popup."],
        ["2", "Fill", "Choose Local/Remote/Dynamic, then fill SSH host, user, bind port, and target."],
        ["3", "Connect", "Connect saves the profile, closes the popup, and returns to Main for status and logs."],
      ];

  const remoteLocalRows = isZh
    ? [
        ["目标", "远程服务器上的 127.0.0.1:8000，本地浏览器也能打开"],
        ["模式", "local"],
        ["SSH 主机", "服务器 IP 或域名，例如 203.0.113.10"],
        ["SSH 端口", "22"],
        ["SSH 用户", "root / ubuntu / deploy"],
        ["监听地址", "127.0.0.1"],
        ["监听端口", "8000"],
        ["目标地址", "127.0.0.1"],
        ["目标端口", "8000"],
        ["打开方式", "连接成功后访问 http://127.0.0.1:8000"],
      ]
    : [
        ["Goal", "Open 127.0.0.1:8000 on the remote server from your local browser"],
        ["Mode", "local"],
        ["SSH Host", "Server IP or domain, for example 203.0.113.10"],
        ["SSH Port", "22"],
        ["SSH User", "root / ubuntu / deploy"],
        ["Bind Host", "127.0.0.1"],
        ["Bind Port", "8000"],
        ["Target Host", "127.0.0.1"],
        ["Target Port", "8000"],
        ["Open", "Visit http://127.0.0.1:8000 after connecting"],
      ];

  const pageCards = isZh
    ? [
        ["主页面", "查看当前连接、运行状态和日志。点击一条记录可选中，再点一次可取消选中。这里也可以新建连接。"],
        ["连接管理", "程序默认打开此页。历史连接集中在这里，新建、配置、连接、一次性密码连接和删除都从这里开始。"],
        ["配置弹窗", "填写或修改 SSH 与端口转发参数。点击连接会自动保存并回到主页面。"],
        ["设置", "切换 dark/light 主题、界面语言和日志等级。Debug 适合排查连接失败。"],
      ]
    : [
        ["Main", "Inspect current connections, status, and logs. Click a row to select it, then click again to clear selection. New profiles can also be created here."],
        ["Connection Management", "The app opens here by default. Create, configure, connect, password-connect, and delete saved profiles from one place."],
        ["Profile Popup", "Fill or edit SSH and forwarding parameters. Connect saves the profile and returns to Main."],
        ["Settings", "Switch dark/light theme, language, and log level. Debug is useful for connection troubleshooting."],
      ];

  const modeCards = isZh
    ? [
        ["Local", "本地端口 -> SSH 服务器能访问到的目标服务。最常用，适合远程数据库、远程 Web 服务。"],
        ["Remote", "远程端口 -> 本机服务。适合临时把本地开发服务暴露给服务器侧访问。"],
        ["Dynamic", "创建 SOCKS 代理。通常只需要监听地址和监听端口。"],
      ]
    : [
        ["Local", "Local port -> a service reachable from the SSH server. Best for remote databases or web services."],
        ["Remote", "Remote port -> a local service. Useful when the server side needs to reach your development service."],
        ["Dynamic", "Create a SOCKS proxy. Usually only bind host and bind port are required."],
      ];

  const tips = isZh
    ? [
        "如果第一次连接某台服务器，先在 PowerShell 手动执行一次 ssh 用户@主机 -p 端口，确认主机指纹。",
        "推荐使用 SSH 密钥或 ssh-agent；一次性密码不会写入历史配置。",
        "关闭应用时，程序会自动清理由它启动的 SSH 转发进程。",
      ]
    : [
        "For first-time hosts, run ssh user@host -p port once in PowerShell to confirm the host key.",
        "SSH keys or ssh-agent are recommended. One-time passwords are not written to saved profiles.",
        "When the app exits, SSH forwarding processes started by the app are cleaned up automatically.",
      ];

  return (
    <div className="grid gap-5">
      <Card>
        <CardHeader>
          <CardTitle>{isZh ? "使用说明" : "Guide"}</CardTitle>
          <CardDescription>
            {isZh
              ? "按实际使用顺序整理：先创建连接，再参考样例填写，最后用日志排查问题。"
              : "Organized by the real workflow: create a profile, fill it from an example, then use logs for troubleshooting."}
          </CardDescription>
        </CardHeader>
        <div className="grid grid-cols-3 gap-4">
          {quickSteps.map(([step, title, body]) => (
            <div key={step} className="rounded-2xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-800 dark:bg-slate-900">
              <div className="flex h-8 w-8 items-center justify-center rounded-full bg-blue-600 text-sm font-semibold text-white">{step}</div>
              <div className="mt-3 text-sm font-semibold text-slate-950 dark:text-slate-50">{title}</div>
              <p className="mt-2 text-sm leading-6 text-slate-500 dark:text-slate-400">{body}</p>
            </div>
          ))}
        </div>
      </Card>

      <WindowExampleCard
        title={isZh ? "常用样例：本地打开远程 127.0.0.1:8000" : "Common Example: Open Remote 127.0.0.1:8000 Locally"}
        description={
          isZh
            ? "这是 Local 转发的典型场景。远程服务只监听服务器自己的 127.0.0.1，但你可以通过 SSH 隧道在本地浏览器访问。"
            : "This is the typical Local forwarding case. The remote service only listens on the server's own 127.0.0.1, but your local browser can reach it through the SSH tunnel."
        }
        rows={remoteLocalRows}
      />

      <div className="grid grid-cols-2 gap-5">
        <GuidePanel title={isZh ? "每个页面怎么用" : "Pages"}>
          {pageCards.map(([title, body]) => (
            <GuideLine key={title} title={title} body={body} />
          ))}
        </GuidePanel>
        <GuidePanel title={isZh ? "转发模式怎么选" : "Forwarding Modes"}>
          {modeCards.map(([title, body]) => (
            <GuideLine key={title} title={title} body={body} />
          ))}
        </GuidePanel>
      </div>

      <GuidePanel title={isZh ? "排错与注意事项" : "Troubleshooting"}>
        <div className="grid gap-3">
          {tips.map((tip) => (
            <div key={tip} className="flex gap-3 rounded-2xl bg-slate-50 p-4 text-sm leading-6 text-slate-600 dark:bg-slate-900 dark:text-slate-300">
              <CheckCircle2 className="mt-0.5 shrink-0 text-blue-600" size={17} />
              <span>{tip}</span>
            </div>
          ))}
        </div>
      </GuidePanel>
    </div>
  );
}

function GuidePanel(props: { title: string; children: ReactNode }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>{props.title}</CardTitle>
      </CardHeader>
      <div className="grid gap-3">{props.children}</div>
    </Card>
  );
}

function GuideLine(props: { title: string; body: string }) {
  return (
    <div className="rounded-2xl border border-slate-200 bg-white p-4 dark:border-slate-800 dark:bg-slate-950">
      <div className="text-sm font-semibold text-slate-950 dark:text-slate-50">{props.title}</div>
      <p className="mt-2 text-sm leading-6 text-slate-500 dark:text-slate-400">{props.body}</p>
    </div>
  );
}

function GuidePage(props: { language: Language }) {
  const windowExamples =
    props.language === "zh-CN"
      ? [
          {
            title: "主页面怎么用",
            description: "这里用来看连接是否运行、查看日志、快速连接或断开。",
            rows: [
              ["当前连接", "选择一条历史连接后查看状态"],
              ["快捷操作", "连接选中项 / 断开选中项 / 全部断开"],
              ["运行日志", "连接失败时先看这里的 error 或 warning"],
            ],
          },
          {
            title: "配置端口怎么填",
            description: "这是最重要的页面，保存后会出现在历史连接里。",
            rows: [
              ["名称", "公司数据库 / 测试服务器"],
              ["SSH 主机", "example.com 或 192.168.1.10"],
              ["SSH 用户", "root / ubuntu / deploy"],
              ["监听地址", "127.0.0.1"],
              ["监听端口", "15432"],
              ["目标地址", "127.0.0.1"],
              ["目标端口", "5432"],
            ],
          },
          {
            title: "设置页面怎么选",
            description: "这些配置会保存到本地，下次启动自动恢复。",
            rows: [
              ["主题", "dark 或 light"],
              ["语言", "中文（简体）或 English"],
              ["日志等级", "排查问题选 debug，日常使用选 info"],
            ],
          },
          {
            title: "历史连接怎么用",
            description: "历史连接用于复用配置，也适合维护多台服务器。",
            rows: [
              ["编辑", "把历史连接载入配置端口页面"],
              ["连接", "启动该条 SSH 转发并跳回主页面"],
              ["删除", "需要先断开运行中的连接"],
            ],
          },
        ]
      : [
          {
            title: "Main page",
            description: "Use it to check tunnel status, inspect logs, and quickly connect or disconnect.",
            rows: [
              ["Current connections", "Select a saved profile and inspect its status"],
              ["Quick actions", "Connect selected / disconnect selected / stop all"],
              ["Runtime logs", "Check error or warning entries when a tunnel fails"],
            ],
          },
          {
            title: "Connection Management",
            description: "The app opens here by default. Click New or Configure to open the parameter popup.",
            rows: [
              ["New", "Open a blank profile popup"],
              ["Configure", "Edit SSH and forwarding parameters"],
              ["Connect", "Start forwarding and return to Main"],
              ["Delete", "Disconnect running tunnels before deletion"],
            ],
          },
          {
            title: "Settings",
            description: "These choices are saved locally and restored on next launch.",
            rows: [
              ["Theme", "dark or light"],
              ["Language", "中文（简体） or English"],
              ["Log Level", "Use debug for troubleshooting, info for daily use"],
            ],
          },
          {
            title: "History",
            description: "Use saved profiles to manage multiple servers and reuse tunnel settings.",
            rows: [
              ["Edit", "Load the profile into Configure Ports"],
              ["Connect", "Start the SSH forwarding process and return to Main"],
              ["Delete", "Disconnect running tunnels before deletion"],
            ],
          },
        ];

  const sections = [
    {
      title: t(props.language, "guideStartTitle"),
      items: [
        t(props.language, "guideStartOne"),
        t(props.language, "guideStartTwo"),
        t(props.language, "guideStartThree"),
      ],
    },
    {
      title: t(props.language, "guideModeTitle"),
      items: [
        t(props.language, "guideModeLocal"),
        t(props.language, "guideModeRemote"),
        t(props.language, "guideModeDynamic"),
      ],
    },
    {
      title: t(props.language, "guideKeepTitle"),
      items: [
        t(props.language, "guideKeepAlive"),
        t(props.language, "guideLogs"),
        t(props.language, "guideDelete"),
      ],
    },
    {
      title: t(props.language, "guidePackageTitle"),
      items: [
        t(props.language, "guidePackageOne"),
        t(props.language, "guidePackageTwo"),
      ],
    },
  ];

  return (
    <div className="grid gap-5">
      <Card>
        <CardHeader>
          <CardTitle>{t(props.language, "guideTitle")}</CardTitle>
          <CardDescription>{t(props.language, "guideDescription")}</CardDescription>
        </CardHeader>
        <div className="grid grid-cols-3 gap-4">
          <InfoTile title={t(props.language, "guideTileConfig")} body={t(props.language, "guideTileConfigBody")} />
          <InfoTile title={t(props.language, "guideTileConnect")} body={t(props.language, "guideTileConnectBody")} />
          <InfoTile title={t(props.language, "guideTileObserve")} body={t(props.language, "guideTileObserveBody")} />
        </div>
      </Card>

      <WindowExampleCard
        title={props.language === "zh-CN" ? "\u5e38\u7528\u6837\u4f8b\uff1a\u5728\u672c\u5730\u6253\u5f00\u8fdc\u7a0b 127.0.0.1:8000" : "Common example: open remote 127.0.0.1:8000 locally"}
        description={
          props.language === "zh-CN"
            ? "\u8fd9\u79cd\u573a\u666f\u9009\u62e9 Local \u6a21\u5f0f\u3002\u8fde\u63a5\u6210\u529f\u540e\uff0c\u5728\u672c\u673a\u6d4f\u89c8\u5668\u6253\u5f00 http://127.0.0.1:8000\uff0c\u5b9e\u9645\u8bbf\u95ee\u7684\u662f SSH \u670d\u52a1\u5668\u4e0a\u7684 127.0.0.1:8000\u3002"
            : "Use Local mode. After connecting, open http://127.0.0.1:8000 in your local browser; it reaches 127.0.0.1:8000 on the SSH server."
        }
        rows={
          props.language === "zh-CN"
            ? [
                ["\u6a21\u5f0f", "local"],
                ["SSH \u4e3b\u673a", "\u4f60\u7684\u670d\u52a1\u5668 IP \u6216\u57df\u540d"],
                ["SSH \u7aef\u53e3", "22"],
                ["SSH \u7528\u6237", "root / ubuntu / deploy"],
                ["\u76d1\u542c\u5730\u5740", "127.0.0.1"],
                ["\u76d1\u542c\u7aef\u53e3", "8000"],
                ["\u76ee\u6807\u5730\u5740", "127.0.0.1"],
                ["\u76ee\u6807\u7aef\u53e3", "8000"],
              ]
            : [
                ["Mode", "local"],
                ["SSH Host", "Your server IP or domain"],
                ["SSH Port", "22"],
                ["SSH User", "root / ubuntu / deploy"],
                ["Bind Host", "127.0.0.1"],
                ["Bind Port", "8000"],
                ["Target Host", "127.0.0.1"],
                ["Target Port", "8000"],
              ]
        }
      />

      <div className="grid grid-cols-2 gap-5">
        {windowExamples.map((example) => (
          <WindowExampleCard key={example.title} title={example.title} description={example.description} rows={example.rows} />
        ))}
      </div>

      <div className="grid grid-cols-2 gap-5">
        {sections.map((section) => (
          <Card key={section.title}>
            <CardHeader>
              <CardTitle>{section.title}</CardTitle>
            </CardHeader>
            <div className="grid gap-3">
              {section.items.map((item) => (
                <div key={item} className="flex gap-3 rounded-2xl bg-slate-50 p-4 text-sm leading-6 text-slate-600 dark:bg-slate-900 dark:text-slate-300">
                  <CheckCircle2 className="mt-0.5 shrink-0 text-blue-600" size={17} />
                  <span>{item}</span>
                </div>
              ))}
            </div>
          </Card>
        ))}
      </div>
    </div>
  );
}

function WindowExampleCard(props: { title: string; description: string; rows: string[][] }) {
  return (
    <div className="rounded-2xl border border-blue-100 bg-white p-5 text-slate-900 shadow-sm transition duration-200 hover:-translate-y-0.5 hover:shadow-soft dark:border-blue-950/60">
      <div className="text-base font-semibold">{props.title}</div>
      <p className="mt-2 text-sm leading-6 text-slate-500">{props.description}</p>
      <div className="mt-4 overflow-hidden rounded-xl border border-slate-200">
        {props.rows.map(([label, value]) => (
          <div key={`${label}-${value}`} className="grid grid-cols-[120px_1fr] border-b border-slate-100 last:border-b-0">
            <div className="bg-blue-50 px-3 py-2 text-sm font-medium text-blue-700">{label}</div>
            <div className="px-3 py-2 text-sm text-slate-600">{value}</div>
          </div>
        ))}
      </div>
    </div>
  );
}

function InfoTile(props: { title: string; body: string }) {
  return (
    <div className="rounded-2xl border border-slate-200 bg-slate-50 p-4 transition duration-200 hover:-translate-y-0.5 hover:bg-blue-50 dark:border-slate-800 dark:bg-slate-900 dark:hover:bg-blue-950/30">
      <div className="text-sm font-semibold text-slate-950 dark:text-slate-50">{props.title}</div>
      <p className="mt-2 text-sm leading-6 text-slate-500 dark:text-slate-400">{props.body}</p>
    </div>
  );
}

function ProfileTable(props: {
  language: Language;
  profiles: TunnelProfile[];
  selectedId: string;
  compact?: boolean;
  onSelect: (profile: TunnelProfile) => void;
  actions?: (profile: TunnelProfile) => ReactNode;
}) {
  return (
    <div className="overflow-hidden rounded-2xl border border-slate-200 dark:border-slate-800">
      <table className="w-full border-collapse text-sm">
        <thead className="bg-slate-100 text-left text-slate-500 dark:bg-slate-900 dark:text-slate-400">
          <tr>
            <th className="px-4 py-3 font-medium">{t(props.language, "name")}</th>
            <th className="px-4 py-3 font-medium">{t(props.language, "mode")}</th>
            <th className="px-4 py-3 font-medium">{t(props.language, "bind")}</th>
            <th className="px-4 py-3 font-medium">{t(props.language, "target")}</th>
            <th className="px-4 py-3 font-medium">{t(props.language, "status")}</th>
            {props.actions && <th className="px-4 py-3 font-medium"></th>}
          </tr>
        </thead>
        <tbody>
          {props.profiles.map((profile) => {
            const isSelected = props.selectedId === profile.id;
            return (
              <tr
                key={profile.id}
                onClick={() => props.onSelect(profile)}
                className={cn(
                  "cursor-pointer border-t border-slate-200 transition duration-200 hover:bg-blue-50/80 dark:border-slate-800 dark:hover:bg-slate-900",
                  isSelected && "bg-blue-100/90 text-blue-950 shadow-[inset_5px_0_0_#2563eb] dark:bg-blue-950/60 dark:text-blue-100",
                )}
              >
                <td className="px-4 py-3 font-semibold">
                  <div className="flex items-center gap-2">
                    {isSelected && <span className="h-2 w-2 rounded-full bg-blue-600" />}
                    {profile.name}
                  </div>
                </td>
                <td className={cn("px-4 py-3", isSelected ? "text-blue-700 dark:text-blue-200" : "text-slate-500")}>{profile.mode}</td>
                <td className={cn("px-4 py-3", isSelected ? "text-blue-700 dark:text-blue-200" : "text-slate-500")}>{profile.bindDisplay}</td>
                <td className={cn("px-4 py-3", isSelected ? "text-blue-700 dark:text-blue-200" : "text-slate-500")}>{profile.targetDisplay}</td>
                <td className="px-4 py-3">
                  <span className={cn("rounded-full px-2.5 py-1 text-xs font-medium", profile.status === "running" ? "bg-emerald-100 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300" : "bg-slate-100 text-slate-500 dark:bg-slate-900 dark:text-slate-400")}>
                    {profile.status === "running" ? t(props.language, "running") : t(props.language, "stopped")}
                  </span>
                </td>
                {props.actions && (
                  <td className="px-4 py-3" onClick={(event) => event.stopPropagation()}>
                    {props.actions(profile)}
                  </td>
                )}
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

function LogTable(props: { language: Language; logs: LogEntry[] }) {
  return (
    <div className="max-h-64 overflow-auto rounded-2xl border border-slate-200 dark:border-slate-800">
      <table className="w-full text-sm">
        <thead className="sticky top-0 bg-slate-100 text-left text-slate-500 dark:bg-slate-900 dark:text-slate-400">
          <tr>
            <th className="px-4 py-3 font-medium">Time</th>
            <th className="px-4 py-3 font-medium">Level</th>
            <th className="px-4 py-3 font-medium">{t(props.language, "message")}</th>
          </tr>
        </thead>
        <tbody>
          {props.logs.map((log, index) => (
            <tr key={`${log.timestamp}-${index}`} className="border-t border-slate-200 dark:border-slate-800">
              <td className="px-4 py-2 text-slate-500">{log.timestamp}</td>
              <td className="px-4 py-2 uppercase text-slate-500">{log.level}</td>
              <td className="px-4 py-2">{log.message}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function Field(props: { label: string; value: string; onChange: (value: string) => void }) {
  return (
    <label className="grid gap-2 text-sm font-medium text-slate-600 dark:text-slate-300">
      {props.label}
      <Input value={props.value} onChange={(event) => props.onChange(event.target.value)} />
    </label>
  );
}

function SettingSelect(props: { label: string; value: string; onChange: (value: string) => void; children: ReactNode }) {
  return (
    <label className="grid grid-cols-[160px_240px] items-center gap-4 text-sm font-medium text-slate-600 dark:text-slate-300">
      {props.label}
      <Select value={props.value} onChange={(event) => props.onChange(event.target.value)}>
        {props.children}
      </Select>
    </label>
  );
}
