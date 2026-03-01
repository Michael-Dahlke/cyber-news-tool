import { useEffect, useState, useCallback } from "react";
import {
  Settings2,
  Loader2,
  Save,
  Database,
  Info,
  AlertCircle,
  CheckCircle2,
} from "lucide-react";
import { cn } from "@/lib/utils";
import * as settingsService from "@/services/settingsService";
import type { AppSettings, SettingsUpdate, Theme, AppInfo } from "@/types";
import { SourceManager } from "./SourceManager";

type TabId = "general" | "sources" | "about";

const tabs: { id: TabId; label: string }[] = [
  { id: "general", label: "General" },
  { id: "sources", label: "Sources" },
  { id: "about", label: "About" },
];

export function SettingsView() {
  const [activeTab, setActiveTab] = useState<TabId>("general");
  const [, setSettings] = useState<AppSettings | null>(null);
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  // Form state
  const [theme, setTheme] = useState<Theme>("system");
  const [refreshInterval, setRefreshInterval] = useState(60);
  const [cacheRetention, setCacheRetention] = useState(30);

  const fetchSettings = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [settingsResult, infoResult] = await Promise.all([
        settingsService.getSettings(),
        settingsService.getAppInfo(),
      ]);
      setSettings(settingsResult);
      setAppInfo(infoResult);
      setTheme(settingsResult.theme);
      setRefreshInterval(settingsResult.feed_refresh_interval_min);
      setCacheRetention(settingsResult.cache_retention_days);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchSettings();
  }, [fetchSettings]);

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    setSuccess(false);
    try {
      const update: SettingsUpdate = {
        theme,
        feed_refresh_interval_min: refreshInterval,
        cache_retention_days: cacheRetention,
      };
      const updated = await settingsService.updateSettings(update);
      setSettings(updated);
      setSuccess(true);

      // Apply theme
      applyTheme(updated.theme);

      setTimeout(() => setSuccess(false), 3000);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      if (msg === "INVALID_INTERVAL") {
        setError("Refresh interval must be at least 1 minute");
      } else if (msg === "INVALID_RETENTION") {
        setError("Cache retention must be at least 1 day");
      } else {
        setError(msg);
      }
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-6 p-6">
      {/* Header */}
      <div className="flex items-center gap-2">
        <Settings2 className="h-6 w-6 text-foreground" />
        <h1 className="text-2xl font-bold text-foreground">Settings</h1>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 border-b border-border">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            type="button"
            onClick={() => setActiveTab(tab.id)}
            className={cn(
              "px-4 py-2 text-sm font-medium transition-colors",
              activeTab === tab.id
                ? "border-b-2 border-primary text-foreground"
                : "text-muted-foreground hover:text-foreground"
            )}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab content */}
      {activeTab === "general" && (
        <GeneralSettings
          theme={theme}
          setTheme={setTheme}
          refreshInterval={refreshInterval}
          setRefreshInterval={setRefreshInterval}
          cacheRetention={cacheRetention}
          setCacheRetention={setCacheRetention}
          error={error}
          success={success}
          saving={saving}
          onSave={handleSave}
        />
      )}

      {activeTab === "sources" && <SourceManager />}

      {activeTab === "about" && appInfo && <AboutSection appInfo={appInfo} />}
    </div>
  );
}

// ─── General Settings Tab ─────────────────────────────────

function GeneralSettings({
  theme,
  setTheme,
  refreshInterval,
  setRefreshInterval,
  cacheRetention,
  setCacheRetention,
  error,
  success,
  saving,
  onSave,
}: {
  theme: Theme;
  setTheme: (t: Theme) => void;
  refreshInterval: number;
  setRefreshInterval: (n: number) => void;
  cacheRetention: number;
  setCacheRetention: (n: number) => void;
  error: string | null;
  success: boolean;
  saving: boolean;
  onSave: () => void;
}) {
  return (
    <div className="flex max-w-lg flex-col gap-6">
      {/* Theme */}
      <div className="flex flex-col gap-1.5">
        <label
          htmlFor="theme"
          className="text-sm font-medium text-foreground"
        >
          Theme
        </label>
        <select
          id="theme"
          value={theme}
          onChange={(e) => setTheme(e.target.value as Theme)}
          className="rounded-md border border-input bg-background px-3 py-2 text-sm focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
        >
          <option value="system">System</option>
          <option value="light">Light</option>
          <option value="dark">Dark</option>
        </select>
        <p className="text-xs text-muted-foreground">
          Choose your preferred appearance
        </p>
      </div>

      {/* Feed refresh interval */}
      <div className="flex flex-col gap-1.5">
        <label
          htmlFor="refresh-interval"
          className="text-sm font-medium text-foreground"
        >
          Feed Refresh Interval (minutes)
        </label>
        <input
          id="refresh-interval"
          type="number"
          min={1}
          value={refreshInterval}
          onChange={(e) => setRefreshInterval(Number(e.target.value))}
          className="rounded-md border border-input bg-background px-3 py-2 text-sm focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
        />
        <p className="text-xs text-muted-foreground">
          How often to automatically check for new articles (default: 60)
        </p>
      </div>

      {/* Cache retention */}
      <div className="flex flex-col gap-1.5">
        <label
          htmlFor="cache-retention"
          className="text-sm font-medium text-foreground"
        >
          Cache Retention (days)
        </label>
        <input
          id="cache-retention"
          type="number"
          min={1}
          value={cacheRetention}
          onChange={(e) => setCacheRetention(Number(e.target.value))}
          className="rounded-md border border-input bg-background px-3 py-2 text-sm focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
        />
        <p className="text-xs text-muted-foreground">
          How long to keep cached article content (default: 30)
        </p>
      </div>

      {/* Status messages */}
      {error && (
        <div className="flex items-center gap-2 rounded-md border border-destructive/20 bg-destructive/5 px-3 py-2 text-sm text-destructive">
          <AlertCircle className="h-4 w-4" />
          {error}
        </div>
      )}
      {success && (
        <div className="flex items-center gap-2 rounded-md border border-general/20 bg-general/5 px-3 py-2 text-sm text-general">
          <CheckCircle2 className="h-4 w-4" />
          Settings saved
        </div>
      )}

      {/* Save button */}
      <div>
        <button
          type="button"
          onClick={onSave}
          disabled={saving}
          className={cn(
            "inline-flex items-center gap-2 rounded-md px-4 py-2 text-sm font-medium transition-colors",
            "bg-primary text-primary-foreground hover:bg-primary/90",
            "disabled:pointer-events-none disabled:opacity-50"
          )}
        >
          {saving ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <Save className="h-4 w-4" />
          )}
          {saving ? "Saving..." : "Save Settings"}
        </button>
      </div>
    </div>
  );
}

// ─── About Tab ────────────────────────────────────────────

function AboutSection({ appInfo }: { appInfo: AppInfo }) {
  return (
    <div className="flex max-w-lg flex-col gap-4">
      <div className="flex items-center gap-2 text-muted-foreground">
        <Info className="h-5 w-5" />
        <span className="text-sm font-medium">Application Info</span>
      </div>

      <div className="rounded-lg border border-border bg-card p-4">
        <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-sm">
          <dt className="text-muted-foreground">Version</dt>
          <dd className="font-mono text-foreground">{appInfo.version}</dd>

          <dt className="text-muted-foreground">Platform</dt>
          <dd className="font-mono text-foreground">{appInfo.os}</dd>

          <dt className="text-muted-foreground">Database</dt>
          <dd className="flex items-center gap-1.5 font-mono text-foreground">
            <Database className="h-3.5 w-3.5 text-muted-foreground" />
            <span className="truncate" title={appInfo.db_path}>
              {appInfo.db_path}
            </span>
          </dd>
        </dl>
      </div>

      <p className="text-xs text-muted-foreground">
        Cybersecurity News Reader — a desktop application for aggregating and
        analyzing cybersecurity news with KQL query generation capabilities.
      </p>
    </div>
  );
}

// ─── Theme Helper ─────────────────────────────────────────

function applyTheme(theme: Theme) {
  const root = document.documentElement;
  if (theme === "dark") {
    root.classList.add("dark");
  } else if (theme === "light") {
    root.classList.remove("dark");
  } else {
    // System preference
    const prefersDark = window.matchMedia(
      "(prefers-color-scheme: dark)"
    ).matches;
    if (prefersDark) {
      root.classList.add("dark");
    } else {
      root.classList.remove("dark");
    }
  }
}
