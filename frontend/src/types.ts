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
  // 跳板机（ProxyJump），可空。形如 user@jump-host:port，多级用逗号分隔。
  proxyJump: string;
  forwards: Forward[];
  pinned: boolean;
  // 视图字段：最后修改时间（Unix 毫秒），列表排序用
  updatedAt?: number;
}

/** 导入结果：status="conflict" 时附带重复主机名，需用户选择覆盖策略。 */
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

/** VS Code 与 Remote-SSH 扩展的安装情况。 */
export interface VscodeStatus {
  installed: boolean;
  remoteSsh: boolean;
}

/** 一条 VS Code Remote-SSH 历史远端文件夹。 */
export interface VscodeHistoryEntry {
  uri: string;
  path: string;
}

/** 直连/打开根目录的结果。 */
export interface VscodeOpenRootResult {
  addedToConfig: boolean;
  alias: string;
}
