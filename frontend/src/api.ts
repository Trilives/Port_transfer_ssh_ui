import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, Forward, Host, LogEntry } from "./types";

export const api = {
  // 主机（一级）
  listHosts: () => invoke<Host[]>("list_hosts"),
  saveHost: (host: Host) => invoke<Host>("save_host", { host }),
  setHostPinned: (id: string, pinned: boolean) => invoke<Host>("set_host_pinned", { id, pinned }),
  deleteHost: (id: string) => invoke<void>("delete_host", { id }),

  // 端口转发（二级）
  saveForward: (hostId: string, forward: Forward) => invoke<Host>("save_forward", { hostId, forward }),
  deleteForward: (hostId: string, forwardId: string) => invoke<Host>("delete_forward", { hostId, forwardId }),
  connectForward: (hostId: string, forwardId: string) => invoke<Host>("connect_forward", { hostId, forwardId }),
  connectForwardWithPassword: (hostId: string, forwardId: string, password: string) =>
    invoke<Host>("connect_forward_with_password", { hostId, forwardId, password }),
  disconnectForward: (hostId: string, forwardId: string) => invoke<Host>("disconnect_forward", { hostId, forwardId }),
  disconnectHost: (hostId: string) => invoke<Host>("disconnect_host", { hostId }),
  disconnectAll: () => invoke<void>("disconnect_all"),

  // 指令 / 终端 / 密钥
  sendCommand: (hostId: string, command: string) => invoke<string>("send_command", { hostId, command }),
  sendCommandWithPassword: (hostId: string, command: string, password: string) =>
    invoke<string>("send_command_with_password", { hostId, command, password }),
  openTerminal: (hostId: string) => invoke<void>("open_terminal", { hostId }),
  uploadPublicKey: (hostId: string, password: string) => invoke<Host>("upload_public_key", { hostId, password }),
  probeConnection: (hostId: string) => invoke<string>("probe_connection", { hostId }),
  getHostFingerprint: (hostId: string) => invoke<string>("get_host_fingerprint", { hostId }),
  removeKnownHost: (hostId: string) => invoke<void>("remove_known_host", { hostId }),

  // 系统环境
  checkSsh: () => invoke<boolean>("check_ssh"),
  installOpenssh: () => invoke<void>("install_openssh"),

  // 设置 / 日志
  getSettings: () => invoke<AppSettings>("get_settings"),
  saveSettings: (settings: AppSettings) => invoke<AppSettings>("save_settings_cmd", { settings }),
  listLogs: (level: string) => invoke<LogEntry[]>("list_logs", { level }),
};
