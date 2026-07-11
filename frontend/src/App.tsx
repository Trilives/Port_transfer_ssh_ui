import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getVersion } from "@tauri-apps/api/app";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { Activity, BookOpen, ScrollText, Server, Settings as SettingsIcon, Terminal } from "lucide-react";
import { api } from "./api";
import { cn, forwardWebUrl } from "./lib/utils";
import { t } from "./i18n";
import {
  CloseBehaviorDialog,
  ConfirmDialog,
  ConnectionErrorDialog,
  CriticalErrorDialog,
  ForwardDialog,
  HostDialog,
  HostKeyChangedDialog,
  ImportConflictDialog,
  InputPasswordDialog,
  KeyUploadDialog,
  OpenHistoryDialog,
  PasswordDialog,
  SelectHostsDialog,
  SendCommandDialog,
  SshMissingDialog,
  VscodeMissingDialog,
} from "./components/dialogs";
import { DashboardPage } from "./pages/DashboardPage";
import { ConfigPage } from "./pages/ConfigPage";
import { LogsPage } from "./pages/LogsPage";
import { SettingsPage } from "./pages/SettingsPage";
import { GuidePage } from "./pages/GuidePage";
import type { AppSettings, CriticalErrorPayload, Forward, HistoryEntry, Host, LogEntry, LogLevel, UpdateState } from "./types";

type Page = "dashboard" | "config" | "logs" | "settings" | "guide";

const defaultSettings: AppSettings = { theme: "light", language: "zh-CN", logLevel: "info", closeBehavior: "ask", autoUpdate: false };

const newHost = (): Host => ({
  id: "",
  name: "",
  sshHost: "",
  sshPort: "22",
  sshUser: "",
  identityFile: "",
  extraOptions: "",
  proxyJump: "",
  forwards: [],
  pinned: false,
});

const newForward = (): Forward => ({
  id: crypto.randomUUID(),
  name: "",
  mode: "local",
  bindHost: "127.0.0.1",
  bindPort: "",
  targetHost: "127.0.0.1",
  targetPort: "",
  keepConnected: true,
});

export function App() {
  const [page, setPage] = useState<Page>("dashboard");
  const [hosts, setHosts] = useState<Host[]>([]);
  const [settings, setSettings] = useState<AppSettings>(defaultSettings);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [expandedIds, setExpandedIds] = useState<Set<string>>(new Set());

  // Dialogs
  const [hostDialog, setHostDialog] = useState<Host | null>(null);
  const [forwardDialog, setForwardDialog] = useState<{ hostId: string; draft: Forward } | null>(null);
  const [sendCmd, setSendCmd] = useState<{ hostId: string; hostName: string } | null>(null);
  const [command, setCommand] = useState("");
  const [commandOutput, setCommandOutput] = useState("");
  const [commandBusy, setCommandBusy] = useState(false);
  const [sendCmdPwOpen, setSendCmdPwOpen] = useState(false);
  const [sendCmdPwValue, setSendCmdPwValue] = useState("");
  const [passwordTarget, setPasswordTarget] = useState<{ host: Host; forward: Forward } | null>(null);
  const [passwordValue, setPasswordValue] = useState("");
  const [keyUploadTarget, setKeyUploadTarget] = useState<Host | null>(null);
  const [keyUploadPassword, setKeyUploadPassword] = useState("");
  const [hostKeyTarget, setHostKeyTarget] = useState<{ host: Host; forward?: Forward; action: "connect" | "upload" } | null>(null);
  const [hostKeyFingerprint, setHostKeyFingerprint] = useState("");
  const [hostKeyFetching, setHostKeyFetching] = useState(false);
  const [criticalError, setCriticalError] = useState<CriticalErrorPayload | null>(null);
  const [connectError, setConnectError] = useState<string | null>(null);
  // Import/export: host-selection dialog and duplicate-conflict dialog.
  const [selectHosts, setSelectHosts] = useState<{
    mode: "import" | "export-file" | "export-config";
    items: Host[];
    selected: Set<string>;
  } | null>(null);
  const [importConflict, setImportConflict] = useState<{
    duplicates: string[];
    hosts: Host[];
    mode: "import" | "export-config";
  } | null>(null);
  const [deleteHostTarget, setDeleteHostTarget] = useState<Host | null>(null);
  const [sshMissing, setSshMissing] = useState(false);
  // Open history: combined VS Code / terminal / port history window + VS Code not-installed prompt.
  const [historyDialog, setHistoryDialog] = useState<{ host: Host; entries: HistoryEntry[] } | null>(null);
  const [vscodeMissing, setVscodeMissing] = useState<"vscode" | "remoteSsh" | null>(null);
  // Close-button prompt (minimize to tray vs quit); `active` = forwards still running.
  const [closePrompt, setClosePrompt] = useState<{ active: boolean } | null>(null);
  const shownCriticalErrors = useRef(new Set<string>());
  // In-app auto-update (Settings page): current version, flow state, and the pending Update handle to install.
  const [appVersion, setAppVersion] = useState("");
  const [update, setUpdate] = useState<UpdateState>({ status: "idle" });
  const [updateDismissed, setUpdateDismissed] = useState(false);
  const pendingUpdate = useRef<Update | null>(null);
  const didStartupUpdateCheck = useRef(false);

  const language = settings.language;
  const modalOpen = Boolean(
    hostDialog || forwardDialog || sendCmd || passwordTarget || keyUploadTarget || hostKeyTarget || criticalError || connectError || selectHosts || importConflict || deleteHostTarget || sshMissing || historyDialog || vscodeMissing || closePrompt,
  );

  useEffect(() => {
    document.documentElement.classList.toggle("dark", settings.theme === "dark");
  }, [settings.theme]);

  // Idle timer only runs on the Config page with no dialog open: 3 minutes of no activity jumps back to Home.
  useEffect(() => {
    if (page !== "config" || modalOpen) return;
    let timer = 0;
    const reset = () => {
      window.clearTimeout(timer);
      timer = window.setTimeout(() => setPage("dashboard"), 3 * 60 * 1000);
    };
    const events = ["mousemove", "mousedown", "keydown", "wheel", "touchstart"];
    events.forEach((event) => window.addEventListener(event, reset, { passive: true }));
    reset();
    return () => {
      window.clearTimeout(timer);
      events.forEach((event) => window.removeEventListener(event, reset));
    };
  }, [page, modalOpen]);

  // Notices auto-dismiss after a few minutes (or can be closed manually); errors persist until closed manually.
  useEffect(() => {
    if (!notice) return;
    const timer = window.setTimeout(() => setNotice(""), 3 * 60 * 1000);
    return () => window.clearTimeout(timer);
  }, [notice]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listen<CriticalErrorPayload>("critical-error", (event) => {
      if (shownCriticalErrors.current.has(event.payload.forwardId)) return;
      shownCriticalErrors.current.add(event.payload.forwardId);
      setCriticalError(event.payload);
    }).then((dispose) => {
      unlisten = dispose;
    });
    return () => unlisten?.();
  }, []);

  // The backend prevents the window close and asks us to prompt (behavior "ask", or "exit" with forwards running).
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listen<boolean>("close-requested", (event) => {
      setClosePrompt({ active: event.payload });
    }).then((dispose) => {
      unlisten = dispose;
    });
    return () => unlisten?.();
  }, []);

  useEffect(() => {
    void api.checkSsh().then((available) => {
      if (!available) setSshMissing(true);
    });
  }, []);

  useEffect(() => {
    void getVersion().then(setAppVersion).catch(() => {});
  }, []);

  // One startup update check: with auto-update on, install silently; otherwise surface a banner on Home.
  useEffect(() => {
    if (didStartupUpdateCheck.current) return;
    didStartupUpdateCheck.current = true;
    void startupUpdateCheck();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    void refreshAll();
    const timer = window.setInterval(() => {
      void refreshHosts();
      void refreshLogs(settings.logLevel);
    }, 1500);
    return () => window.clearInterval(timer);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [settings.logLevel]);

  async function refreshAll() {
    try {
      const [nextSettings, nextHosts] = await Promise.all([api.getSettings(), api.listHosts()]);
      setSettings(nextSettings);
      setHosts(nextHosts);
      await refreshLogs(nextSettings.logLevel);
    } catch (err) {
      setError(String(err));
    }
  }

  async function refreshHosts() {
    try {
      setHosts(await api.listHosts());
    } catch (err) {
      setError(String(err));
    }
  }

  async function refreshLogs(level: LogLevel) {
    try {
      setLogs(await api.listLogs(level));
    } catch {
      /* ignore transient */
    }
  }

  async function updateSettings(next: Partial<AppSettings>) {
    try {
      setSettings(await api.saveSettings({ ...settings, ...next }));
    } catch (err) {
      setError(String(err));
    }
  }

  function toggleExpand(hostId: string) {
    setExpandedIds((prev) => {
      const next = new Set(prev);
      if (next.has(hostId)) next.delete(hostId);
      else next.add(hostId);
      return next;
    });
  }

  function expand(hostId: string) {
    setExpandedIds((prev) => new Set(prev).add(hostId));
  }

  // ---- Host CRUD ----
  async function saveHost() {
    if (!hostDialog) return;
    try {
      const saved = await api.saveHost(hostDialog);
      setHostDialog(null);
      setError("");
      await refreshHosts();
      expand(saved.id);
    } catch (err) {
      setError(String(err));
    }
  }

  async function toggleHostPin(host: Host) {
    try {
      await api.setHostPinned(host.id, !host.pinned);
      await refreshHosts();
    } catch (err) {
      setError(String(err));
    }
  }

  // ---- Import / Export ----
  function openSelectHosts(mode: "import" | "export-file" | "export-config", items: Host[]) {
    setError("");
    setSelectHosts({ mode, items, selected: new Set(items.map((h) => h.id)) });
  }

  async function importFromFile() {
    try {
      const parsed = await api.readImportFile();
      if (parsed.length === 0) return; // user canceled or the file was empty
      openSelectHosts("import", parsed);
    } catch (err) {
      setError(String(err));
    }
  }

  async function importFromConfig() {
    try {
      openSelectHosts("import", await api.readImportSshConfig());
    } catch (err) {
      setError(String(err));
    }
  }

  function toggleSelectHost(id: string) {
    setSelectHosts((prev) => {
      if (!prev) return prev;
      const selected = new Set(prev.selected);
      if (selected.has(id)) selected.delete(id);
      else selected.add(id);
      return { ...prev, selected };
    });
  }

  async function confirmSelectHosts() {
    if (!selectHosts) return;
    const { mode, items, selected } = selectHosts;
    const chosen = items.filter((h) => selected.has(h.id));
    setSelectHosts(null);
    try {
      if (mode === "import") {
        const result = await api.importHosts(chosen, "");
        if (result.status === "conflict") {
          setImportConflict({ duplicates: result.duplicates, hosts: chosen, mode: "import" });
          return;
        }
        await finishImport(result);
      } else if (mode === "export-file") {
        const saved = await api.exportHostsToFile(chosen.map((h) => h.id));
        if (saved) setNotice(t(language, "exportDone"));
      } else {
        const result = await api.exportHostsToSshConfig(chosen.map((h) => h.id), "");
        if (result.status === "conflict") {
          setImportConflict({ duplicates: result.duplicates, hosts: chosen, mode: "export-config" });
          return;
        }
        setNotice(t(language, "exportConfigDone"));
      }
    } catch (err) {
      setError(String(err));
    }
  }

  async function applyImportStrategy(strategy: "overwrite" | "skip") {
    if (!importConflict) return;
    const { hosts, mode } = importConflict;
    setImportConflict(null);
    try {
      if (mode === "export-config") {
        await api.exportHostsToSshConfig(hosts.map((h) => h.id), strategy);
        setNotice(t(language, "exportConfigDone"));
      } else {
        await finishImport(await api.importHosts(hosts, strategy));
      }
    } catch (err) {
      setError(String(err));
    }
  }

  async function finishImport(result: { added: number; overwritten: number; skipped: number }) {
    await refreshHosts();
    setNotice(
      t(language, "importDone")
        .replace("{added}", String(result.added))
        .replace("{overwritten}", String(result.overwritten))
        .replace("{skipped}", String(result.skipped)),
    );
  }

  async function confirmDeleteHost() {
    if (!deleteHostTarget) return;
    const host = deleteHostTarget;
    setDeleteHostTarget(null);
    try {
      await api.deleteHost(host.id);
      await refreshHosts();
    } catch (err) {
      setError(String(err));
    }
  }

  // ---- Forward CRUD ----
  async function saveForward() {
    if (!forwardDialog) return;
    const { hostId, draft } = forwardDialog;
    try {
      const updatedHost = await api.saveForward(hostId, draft);
      setForwardDialog(null);
      setError("");
      await refreshHosts();
      expand(hostId);
      // If "keep connected" is checked and this forward isn't currently running, auto-start it after saving
      // (edits to a running forward are already reconnected by the backend, so skip to avoid double-connecting).
      if (draft.keepConnected) {
        const forward = updatedHost.forwards.find((item) => item.id === draft.id);
        if (forward && forward.status !== "running") await connectForward(updatedHost, forward);
      }
    } catch (err) {
      setError(String(err));
    }
  }

  async function deleteForward(host: Host, forward: Forward) {
    try {
      await api.deleteForward(host.id, forward.id);
      await refreshHosts();
    } catch (err) {
      setError(String(err));
    }
  }

  // ---- Connect flow ----
  async function connectForward(host: Host, forward: Forward) {
    setError("");
    setNotice(t(language, "detectingConnection"));
    try {
      const status = await api.probeConnection(host.id);
      setNotice("");
      if (status === "host_key_changed") {
        await openHostKeyDialog(host, "connect", forward);
        return;
      }
      if (status === "password_required") {
        setPasswordTarget({ host, forward });
        setPasswordValue("");
        return;
      }
      await api.connectForward(host.id, forward.id);
      await refreshHosts();
    } catch (err) {
      setNotice("");
      // Probe/connect failed (unreachable, bad IP/port, network, etc.): show the specific reason in a dialog.
      setConnectError(String(err));
    }
  }

  async function connectWithPassword() {
    if (!passwordTarget || !passwordValue) return;
    try {
      await api.connectForwardWithPassword(passwordTarget.host.id, passwordTarget.forward.id, passwordValue);
      setPasswordTarget(null);
      setPasswordValue("");
      setError("");
      await refreshHosts();
    } catch (err) {
      setError(String(err));
    }
  }

  async function disconnectForward(host: Host, forward: Forward) {
    try {
      await api.disconnectForward(host.id, forward.id);
      await refreshHosts();
    } catch (err) {
      setError(String(err));
    }
  }

  // ---- Key upload flow (host level) ----
  async function requestKeyUpload(host: Host) {
    setError("");
    setNotice(t(language, "detectingConnection"));
    try {
      const status = await api.probeConnection(host.id);
      setNotice("");
      if (status === "ready") {
        setNotice(t(language, "keyUploadNotNeeded"));
        return;
      }
      if (status === "host_key_changed") {
        await openHostKeyDialog(host, "upload");
        return;
      }
      setKeyUploadTarget(host);
      setKeyUploadPassword("");
    } catch (err) {
      setNotice("");
      setConnectError(String(err));
    }
  }

  async function uploadKeyWithPassword() {
    if (!keyUploadTarget || !keyUploadPassword) return;
    try {
      await api.uploadPublicKey(keyUploadTarget.id, keyUploadPassword);
      setKeyUploadTarget(null);
      setKeyUploadPassword("");
      setError("");
    } catch (err) {
      setError(String(err));
    }
  }

  // ---- Host key changed flow ----
  async function openHostKeyDialog(host: Host, action: "connect" | "upload", forward?: Forward) {
    setHostKeyTarget({ host, action, forward });
    setHostKeyFingerprint("");
    setHostKeyFetching(true);
    try {
      setHostKeyFingerprint(await api.getHostFingerprint(host.id));
    } catch {
      setHostKeyFingerprint("");
    } finally {
      setHostKeyFetching(false);
    }
  }

  async function trustHostKeyAndRetry() {
    if (!hostKeyTarget) return;
    const { host, forward, action } = hostKeyTarget;
    setHostKeyTarget(null);
    try {
      await api.removeKnownHost(host.id);
      if (action === "connect" && forward) {
        await connectForward(host, forward);
      } else {
        await requestKeyUpload(host);
      }
    } catch (err) {
      setError(String(err));
    }
  }

  // ---- Send command ----
  async function runSendCommand() {
    if (!sendCmd || !command.trim()) return;
    setCommandBusy(true);
    setCommandOutput("");
    try {
      setCommandOutput(await api.sendCommand(sendCmd.hostId, command));
    } catch (err) {
      setCommandOutput(String(err));
    } finally {
      setCommandBusy(false);
    }
  }

  async function runSendCommandWithPassword() {
    if (!sendCmd || !command.trim() || !sendCmdPwValue) return;
    const password = sendCmdPwValue;
    setSendCmdPwOpen(false);
    setSendCmdPwValue("");
    setCommandBusy(true);
    setCommandOutput("");
    try {
      setCommandOutput(await api.sendCommandWithPassword(sendCmd.hostId, command, password));
    } catch (err) {
      setCommandOutput(String(err));
    } finally {
      setCommandBusy(false);
    }
  }

  function openSendCommand(host: Host) {
    setSendCmd({ hostId: host.id, hostName: host.name });
    setCommand("");
    setCommandOutput("");
    setSendCmdPwOpen(false);
    setSendCmdPwValue("");
  }

  async function openTerminal(host: Host) {
    try {
      await api.openTerminal(host.id);
    } catch (err) {
      setError(String(err));
    }
  }

  async function openForwardWeb(forward: Forward) {
    try {
      await api.openUrl(forwardWebUrl(forward));
    } catch (err) {
      setError(String(err));
    }
  }

  // ---- Open history (VS Code / terminal / port) ----
  async function openVscode(host: Host) {
    setError("");
    try {
      const status = await api.vscodeStatus();
      if (!status.installed) {
        setVscodeMissing("vscode");
        return;
      }
      if (!status.remoteSsh) {
        setVscodeMissing("remoteSsh");
        return;
      }
      // Loading history also rescans VS Code's own history and merges anything new (backend side).
      const entries = await api.listHistory(host.id);
      setHistoryDialog({ host, entries });
    } catch (err) {
      setError(String(err));
    }
  }

  async function openHistoryEntry(entry: HistoryEntry) {
    if (!historyDialog) return;
    const hostId = historyDialog.host.id;
    setHistoryDialog(null);
    try {
      if (entry.kind === "vscode") {
        await api.vscodeOpen(entry.uri, hostId, entry.label);
      } else if (entry.kind === "terminal") {
        await api.openTerminal(hostId);
      } else if (entry.kind === "port" && entry.detail) {
        await api.openUrl(entry.detail);
      }
    } catch (err) {
      setError(String(err));
    }
  }

  async function openVscodeDirect(host: Host) {
    try {
      const result = await api.vscodeOpenDirect(host.id);
      setHistoryDialog(null);
      if (result.addedToConfig) {
        setNotice(t(language, "vscodeAddedToConfig").replace("{alias}", result.alias));
      }
    } catch (err) {
      setHistoryDialog(null);
      setError(String(err));
    }
  }

  async function openVscodePath(host: Host, path: string) {
    if (!path.trim()) return;
    try {
      const result = await api.vscodeOpenPath(host.id, path.trim());
      setHistoryDialog(null);
      if (result.addedToConfig) {
        setNotice(t(language, "vscodeAddedToConfig").replace("{alias}", result.alias));
      }
    } catch (err) {
      setHistoryDialog(null);
      setError(String(err));
    }
  }

  // ---- Close-button prompt (minimize to tray vs quit) ----
  async function minimizeToTray(remember: boolean) {
    setClosePrompt(null);
    if (remember) await updateSettings({ closeBehavior: "minimize" });
    await api.hideToTray();
  }

  async function exitApp(remember: boolean) {
    if (remember) await updateSettings({ closeBehavior: "exit" });
    setClosePrompt(null);
    await api.quitApp();
  }

  async function installSsh() {
    setSshMissing(false);
    try {
      await api.installOpenssh();
      setNotice(t(language, "sshInstallStarted"));
    } catch (err) {
      setError(String(err));
    }
  }

  // ---- In-app auto-update ----
  // Runs once at startup. Reads the freshest autoUpdate setting (state may still be default at mount),
  // then either installs automatically or leaves an "available" update for the Home banner / Settings.
  async function startupUpdateCheck() {
    try {
      const found = await check();
      if (!found) return;
      pendingUpdate.current = found;
      const auto = (await api.getSettings()).autoUpdate;
      if (auto) {
        await installUpdate();
      } else {
        setUpdateDismissed(false);
        setUpdate({ status: "available", version: found.version, notes: found.body ?? "" });
      }
    } catch {
      /* Silent on startup; the user can still check manually in Settings. */
    }
  }

  async function checkForUpdate() {
    setUpdate({ status: "checking" });
    try {
      const found = await check();
      if (found) {
        pendingUpdate.current = found;
        setUpdateDismissed(false);
        setUpdate({ status: "available", version: found.version, notes: found.body ?? "" });
      } else {
        pendingUpdate.current = null;
        setUpdate({ status: "uptodate" });
      }
    } catch (err) {
      setUpdate({ status: "error", error: String(err) });
    }
  }

  async function installUpdate() {
    const found = pendingUpdate.current;
    if (!found) return;
    setUpdate({ status: "downloading", version: found.version });
    try {
      await found.downloadAndInstall();
      setUpdate({ status: "restarting", version: found.version });
      await relaunch();
    } catch (err) {
      setUpdate({ status: "error", error: String(err) });
    }
  }

  const nav = [
    { id: "dashboard" as const, label: t(language, "dashboard"), icon: Activity },
    { id: "config" as const, label: t(language, "config"), icon: Server },
    { id: "logs" as const, label: t(language, "logs"), icon: ScrollText },
    { id: "settings" as const, label: t(language, "settings"), icon: SettingsIcon },
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
            <div className="mb-4 flex items-start justify-between gap-3 rounded-2xl border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-700 dark:border-rose-950 dark:bg-rose-950/40 dark:text-rose-200">
              <span className="whitespace-pre-wrap">{error}</span>
              <button onClick={() => setError("")} className="shrink-0 text-rose-400 hover:text-rose-600">✕</button>
            </div>
          )}
          {notice && (
            <div className="mb-4 flex items-start justify-between gap-3 rounded-2xl border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-700 dark:border-emerald-950 dark:bg-emerald-950/40 dark:text-emerald-200">
              <span className="whitespace-pre-wrap">{notice}</span>
              <button onClick={() => setNotice("")} className="shrink-0 text-emerald-400 hover:text-emerald-600">✕</button>
            </div>
          )}
          {page === "dashboard" && update.status === "available" && !updateDismissed && (
            <div className="mb-4 flex items-center justify-between gap-3 rounded-2xl border border-blue-200 bg-blue-50 px-4 py-3 text-sm text-blue-700 dark:border-blue-950 dark:bg-blue-950/40 dark:text-blue-200">
              <span className="whitespace-pre-wrap">{t(language, "updateBannerText").replace("{version}", update.version ?? "")}</span>
              <div className="flex shrink-0 items-center gap-2">
                <button onClick={installUpdate} className="rounded-lg bg-blue-600 px-3 py-1.5 font-medium text-white transition hover:bg-blue-500">
                  {t(language, "updateNow")}
                </button>
                <button onClick={() => setUpdateDismissed(true)} className="text-blue-400 hover:text-blue-600">✕</button>
              </div>
            </div>
          )}
          <div className="animate-[fadeIn_220ms_ease-out]">
            {page === "dashboard" && (
              <DashboardPage
                language={language}
                hosts={hosts}
                logs={logs}
                onNew={() => {
                  setPage("config");
                  setHostDialog(newHost());
                }}
                onStopAll={async () => {
                  await api.disconnectAll();
                  await refreshHosts();
                }}
                onDisconnectForward={disconnectForward}
                onOpenForwardWeb={(_host, forward) => openForwardWeb(forward)}
                onOpenTerminal={openTerminal}
                onOpenVscode={openVscode}
              />
            )}
            {page === "config" && (
              <ConfigPage
                language={language}
                hosts={hosts}
                expandedIds={expandedIds}
                onToggle={toggleExpand}
                onNewHost={() => setHostDialog(newHost())}
                onEditHost={(host) => setHostDialog(host)}
                onDeleteHost={(host) => setDeleteHostTarget(host)}
                onTogglePin={toggleHostPin}
                onSendCommand={openSendCommand}
                onOpenTerminal={openTerminal}
                onOpenVscode={openVscode}
                onUploadKey={requestKeyUpload}
                onNewForward={(host) => setForwardDialog({ hostId: host.id, draft: newForward() })}
                onConnectForward={connectForward}
                onDisconnectForward={disconnectForward}
                onOpenForwardWeb={(_host, forward) => openForwardWeb(forward)}
                onEditForward={(host, forward) => setForwardDialog({ hostId: host.id, draft: forward })}
                onDeleteForward={deleteForward}
                onImportFromFile={importFromFile}
                onImportFromConfig={importFromConfig}
                onExportToFile={() => openSelectHosts("export-file", hosts)}
                onExportToConfig={() => openSelectHosts("export-config", hosts)}
              />
            )}
            {page === "logs" && <LogsPage language={language} logs={logs} />}
            {page === "settings" && (
              <SettingsPage
                settings={settings}
                setSettings={updateSettings}
                appVersion={appVersion}
                update={update}
                onCheckUpdate={checkForUpdate}
                onInstallUpdate={installUpdate}
              />
            )}
            {page === "guide" && <GuidePage language={language} />}
          </div>
        </section>
      </div>

      {hostDialog && (
        <HostDialog
          language={language}
          draft={hostDialog}
          setDraft={setHostDialog}
          onClose={() => setHostDialog(null)}
          onSave={saveHost}
        />
      )}
      {forwardDialog && (
        <ForwardDialog
          language={language}
          draft={forwardDialog.draft}
          setDraft={(draft) => setForwardDialog({ ...forwardDialog, draft })}
          onClose={() => setForwardDialog(null)}
          onSave={saveForward}
        />
      )}
      {sendCmd && (
        <SendCommandDialog
          language={language}
          hostName={sendCmd.hostName}
          command={command}
          setCommand={setCommand}
          output={commandOutput}
          busy={commandBusy}
          onClose={() => {
            setSendCmd(null);
            setSendCmdPwOpen(false);
            setSendCmdPwValue("");
          }}
          onRun={runSendCommand}
          onRunWithPassword={() => {
            setSendCmdPwValue("");
            setSendCmdPwOpen(true);
          }}
        />
      )}
      {sendCmd && sendCmdPwOpen && (
        <InputPasswordDialog
          language={language}
          title={t(language, "sendWithPassword")}
          description={sendCmd.hostName}
          submitLabel={t(language, "run")}
          password={sendCmdPwValue}
          setPassword={setSendCmdPwValue}
          onCancel={() => {
            setSendCmdPwOpen(false);
            setSendCmdPwValue("");
          }}
          onSubmit={runSendCommandWithPassword}
        />
      )}
      {passwordTarget && (
        <PasswordDialog
          language={language}
          targetName={`${passwordTarget.host.name} / ${passwordTarget.forward.name}`}
          password={passwordValue}
          setPassword={setPasswordValue}
          onCancel={() => {
            setPasswordTarget(null);
            setPasswordValue("");
          }}
          onSubmit={connectWithPassword}
        />
      )}
      {keyUploadTarget && (
        <KeyUploadDialog
          language={language}
          hostName={keyUploadTarget.name}
          password={keyUploadPassword}
          setPassword={setKeyUploadPassword}
          onCancel={() => {
            setKeyUploadTarget(null);
            setKeyUploadPassword("");
          }}
          onSubmit={uploadKeyWithPassword}
        />
      )}
      {hostKeyTarget && (
        <HostKeyChangedDialog
          language={language}
          hostName={hostKeyTarget.host.name}
          fingerprint={hostKeyFingerprint}
          fetching={hostKeyFetching}
          onCancel={() => setHostKeyTarget(null)}
          onTrust={trustHostKeyAndRetry}
        />
      )}
      {criticalError && (
        <CriticalErrorDialog language={language} error={criticalError} onClose={() => setCriticalError(null)} />
      )}
      {connectError && (
        <ConnectionErrorDialog language={language} message={connectError} onClose={() => setConnectError(null)} />
      )}
      {selectHosts && (
        <SelectHostsDialog
          language={language}
          title={t(language, selectHosts.mode === "import" ? "selectHostsToImport" : "selectHostsToExport")}
          confirmLabel={t(language, selectHosts.mode === "import" ? "confirmImport" : "confirmExport")}
          items={selectHosts.items.map((h) => ({ id: h.id, name: h.name, sshHost: h.sshHost }))}
          selected={selectHosts.selected}
          onToggle={toggleSelectHost}
          onSelectAll={() => setSelectHosts((prev) => (prev ? { ...prev, selected: new Set(prev.items.map((h) => h.id)) } : prev))}
          onClearAll={() => setSelectHosts((prev) => (prev ? { ...prev, selected: new Set() } : prev))}
          onCancel={() => setSelectHosts(null)}
          onConfirm={confirmSelectHosts}
        />
      )}
      {importConflict && (
        <ImportConflictDialog
          language={language}
          duplicates={importConflict.duplicates}
          description={importConflict.mode === "export-config" ? t(language, "exportConfigConflictDesc") : undefined}
          onCancel={() => setImportConflict(null)}
          onOverwrite={() => applyImportStrategy("overwrite")}
          onSkip={() => applyImportStrategy("skip")}
        />
      )}
      {deleteHostTarget && (
        <ConfirmDialog
          language={language}
          title={t(language, "confirmDeleteHostTitle")}
          description={`${deleteHostTarget.name} — ${t(language, "confirmDeleteHostDesc")}`}
          confirmLabel={t(language, "delete")}
          onCancel={() => setDeleteHostTarget(null)}
          onConfirm={confirmDeleteHost}
        />
      )}
      {sshMissing && (
        <SshMissingDialog language={language} onCancel={() => setSshMissing(false)} onInstall={installSsh} />
      )}
      {historyDialog && (
        <OpenHistoryDialog
          language={language}
          hostName={historyDialog.host.name}
          entries={historyDialog.entries}
          onOpenEntry={openHistoryEntry}
          onOpenDirect={() => openVscodeDirect(historyDialog.host)}
          onOpenPath={(path) => openVscodePath(historyDialog.host, path)}
          onCancel={() => setHistoryDialog(null)}
        />
      )}
      {vscodeMissing && (
        <VscodeMissingDialog language={language} kind={vscodeMissing} onClose={() => setVscodeMissing(null)} />
      )}
      {closePrompt && (
        <CloseBehaviorDialog
          language={language}
          active={closePrompt.active}
          onMinimize={minimizeToTray}
          onExit={exitApp}
          onCancel={() => setClosePrompt(null)}
        />
      )}
    </main>
  );
}
