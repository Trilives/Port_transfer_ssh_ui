export type TunnelMode = "local" | "remote" | "dynamic";
export type TunnelStatus = "running" | "stopped";
export type ThemeName = "dark" | "light";
export type Language = "zh-CN" | "en-US";
export type LogLevel = "debug" | "info" | "warning" | "error";

/** Level 2: a single port forward (including view fields attached by the backend). */
export interface Forward {
  id: string;
  name: string;
  mode: TunnelMode;
  bindHost: string;
  bindPort: string;
  targetHost: string;
  targetPort: string;
  keepConnected: boolean;
  // View fields (attached when list/save returns)
  status?: TunnelStatus;
  bindDisplay?: string;
  targetDisplay?: string;
}

/** Level 1: an SSH server and its list of forwards. */
export interface Host {
  id: string;
  name: string;
  sshHost: string;
  sshPort: string;
  sshUser: string;
  identityFile: string;
  extraOptions: string;
  // Jump host (ProxyJump), optional. Shaped like user@jump-host:port; use commas for multiple hops.
  proxyJump: string;
  forwards: Forward[];
  pinned: boolean;
  // View field: last-modified time (Unix ms), used for list sorting
  updatedAt?: number;
}

/** Import result: when status="conflict", includes the duplicate host names and requires the user to choose an overwrite strategy. */
export interface ImportResult {
  status: "done" | "conflict";
  duplicates: string[];
  added: number;
  overwritten: number;
  skipped: number;
}

export interface AppSettings {
  theme: ThemeName;
  language: Language;
  logLevel: LogLevel;
}

export interface LogEntry {
  level: LogLevel;
  message: string;
  timestamp: string;
}

export interface CriticalErrorPayload {
  hostId: string;
  forwardId: string;
  name: string;
  message: string;
}

/** Install status of VS Code and the Remote-SSH extension. */
export interface VscodeStatus {
  installed: boolean;
  remoteSsh: boolean;
}

/** A single VS Code Remote-SSH history remote folder entry. */
export interface VscodeHistoryEntry {
  uri: string;
  path: string;
}

/** Result of a direct connect / open-root-directory action. */
export interface VscodeOpenRootResult {
  addedToConfig: boolean;
  alias: string;
}
