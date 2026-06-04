import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, LogEntry, TunnelProfile } from "./types";

export const api = {
  listProfiles: () => invoke<TunnelProfile[]>("list_profiles"),
  saveProfile: (profile: TunnelProfile) => invoke<TunnelProfile>("save_profile", { profile }),
  deleteProfile: (id: string) => invoke<void>("delete_profile", { id }),
  connectProfile: (id: string) => invoke<TunnelProfile>("connect_profile", { id }),
  connectProfileWithPassword: (id: string, password: string) => invoke<TunnelProfile>("connect_profile_with_password", { id, password }),
  uploadPublicKey: (id: string, password: string) => invoke<TunnelProfile>("upload_public_key", { id, password }),
  probeConnection: (id: string) => invoke<string>("probe_connection", { id }),
  getHostFingerprint: (id: string) => invoke<string>("get_host_fingerprint", { id }),
  removeKnownHost: (id: string) => invoke<void>("remove_known_host", { id }),
  disconnectProfile: (id: string) => invoke<TunnelProfile>("disconnect_profile", { id }),
  disconnectAll: () => invoke<void>("disconnect_all"),
  getSettings: () => invoke<AppSettings>("get_settings"),
  saveSettings: (settings: AppSettings) => invoke<AppSettings>("save_settings_cmd", { settings }),
  listLogs: (level: string) => invoke<LogEntry[]>("list_logs", { level }),
};
