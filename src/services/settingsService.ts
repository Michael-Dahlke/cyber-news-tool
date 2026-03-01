import { invoke } from "@tauri-apps/api/core";
import type { AppInfo, AppSettings, SettingsUpdate } from "@/types";

// ─── Settings Commands ────────────────────────────────────

export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("settings_get_all");
}

export async function updateSettings(
  update: SettingsUpdate
): Promise<AppSettings> {
  return invoke<AppSettings>("settings_update", { update });
}

export async function setApiKey(apiKey: string): Promise<boolean> {
  return invoke<boolean>("settings_set_api_key", { apiKey });
}

export async function clearApiKey(): Promise<boolean> {
  return invoke<boolean>("settings_clear_api_key");
}

export async function getAppInfo(): Promise<AppInfo> {
  return invoke<AppInfo>("settings_get_app_info");
}
