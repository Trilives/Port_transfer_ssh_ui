export type TunnelMode = "local" | "remote" | "dynamic";
export type TunnelStatus = "running" | "stopped";
export type ThemeName = "dark" | "light";
export type Language = "zh-CN" | "en-US";
export type LogLevel = "debug" | "info" | "warning" | "error";

/** 二级目录：一条端口转发（含后端附带的视图字段）。 */
export interface Forward {
  id: string;
  name: string;
  mode: TunnelMode;
  bindHost: string;
  bindPort: string;
  targetHost: string;
  targetPort: string;
  keepConnected: boolean;
  // 视图字段（list/save 返回时附带）
  status?: TunnelStatus;
  bindDisplay?: string;
  targetDisplay?: string;
}

/** 一级目录：一台 SSH 服务器及其下的转发列表。 */
export interface Host {
  id: string;
  name: string;
  sshHost: string;
  sshPort: string;
  sshUser: string;
  identityFile: string;
  extraOptions: string;
  forwards: Forward[];
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
