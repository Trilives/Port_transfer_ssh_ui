export type TunnelMode = "local" | "remote" | "dynamic";
export type TunnelStatus = "running" | "stopped";
export type ThemeName = "dark" | "light";
export type Language = "zh-CN" | "en-US";
export type LogLevel = "debug" | "info" | "warning" | "error";

export interface TunnelProfile {
  id: string;
  name: string;
  mode: TunnelMode;
  sshHost: string;
  sshPort: string;
  sshUser: string;
  identityFile: string;
  bindHost: string;
  localPort: string;
  remoteHost: string;
  remotePort: string;
  extraOptions: string;
  keepConnected: boolean;
  status?: TunnelStatus;
  bindDisplay?: string;
  targetDisplay?: string;
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
