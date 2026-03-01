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
import * as articleService from "@/services/articleService";
import type { AppSettings, SettingsUpdate, Theme, AppInfo, CacheStatusResponse } from "@/types";
import { SourceManager } from "./SourceManager";
import { LLMConfig } from "./LLMConfig";
import { useThemeContext } from "@/hooks/ThemeContext";

type TabId = "general" | "llm" | "sources" | "about";

const tabs: { id: TabId; label: string }[] = [
  { id: "general", label: "General" },
  { id: "llm", label: "LLM" },
  { id: "sources", label: "Sources" },
  { id: "about", label: "About" },
];

export function SettingsView() {
  const [activeTab, setActiveTab] = useState<TabId>("general");
  const [currentSettings, setSettings] = useState<AppSettings | null>(null);
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const [cacheStatus, setCacheStatus] = useState<CacheStatusResponse | null>(null);
  const { setTheme: applyTheme } = useThemeContext();

  // Form state
  const [theme, setTheme] = useState<Theme>("system");
  const [refreshThreshold, setRefreshThreshold] = useState(4);
  const [cacheRetention, setCacheRetention] = useState(30);
  const [cacheSizeLimit, setCacheSizeLimit] = useState(20);
  const [readerFontSize, setReaderFontSize] = useState(16);

  const fetchSettings = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [settingsResult, infoResult, cacheResult] = await Promise.all([
        settingsService.getSettings(),
        settingsService.getAppInfo(),
        articleService.getCacheStatus(),
      ]);
      setSettings(settingsResult);
      setAppInfo(infoResult);
      setCacheStatus(cacheResult);
      setTheme(settingsResult.theme);
      setRefreshThreshold(settingsResult.background_refresh_threshold_hours);
      setCacheRetention(settingsResult.cache_retention_days);
      setCacheSizeLimit(settingsResult.article_cache_size_limit_gb);
      setReaderFontSize(settingsResult.reader_font_size);
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
        background_refresh_threshold_hours: refreshThreshold,
        cache_retention_days: cacheRetention,
        article_cache_size_limit_gb: cacheSizeLimit,
        reader_font_size: readerFontSize,
      };
      const updated = await settingsService.updateSettings(update);
      setSettings(updated);
      setSuccess(true);

      // Apply theme via context (propagates to AppShell)
      applyTheme(updated.theme);

      setTimeout(() => setSuccess(false), 3000);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      if (msg === "INVALID_THRESHOLD") {
        setError("Refresh threshold must be at least 1 hour");
      } else if (msg === "INVALID_RETENTION") {
        setError("Cache retention must be at least 1 day");
      } else if (msg === "INVALID_CACHE_LIMIT") {
        setError("Cache size limit must be at least 1 GB");
      } else if (msg === "INVALID_FONT_SIZE") {
        setError("Font size must be between 8 and 72");
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
          refreshThreshold={refreshThreshold}
          setRefreshThreshold={setRefreshThreshold}
          cacheRetention={cacheRetention}
          setCacheRetention={setCacheRetention}
          cacheSizeLimit={cacheSizeLimit}
          setCacheSizeLimit={setCacheSizeLimit}
          readerFontSize={readerFontSize}
          setReaderFontSize={setReaderFontSize}
          cacheStatus={cacheStatus}
          error={error}
          success={success}
          saving={saving}
          onSave={handleSave}
        />
      )}

      {activeTab === "llm" && currentSettings && (
        <div className="max-w-lg">
          <LLMConfig
            settings={currentSettings}
            onSettingsUpdated={(updated) => {
              setSettings(updated);
            }}
          />
        </div>
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
  refreshThreshold,
  setRefreshThreshold,
  cacheRetention,
  setCacheRetention,
  cacheSizeLimit,
  setCacheSizeLimit,
  readerFontSize,
  setReaderFontSize,
  cacheStatus,
  error,
  success,
  saving,
  onSave,
}: {
  theme: Theme;
  setTheme: (t: Theme) => void;
  refreshThreshold: number;
  setRefreshThreshold: (n: number) => void;
  cacheRetention: number;
  setCacheRetention: (n: number) => void;
  cacheSizeLimit: number;
  setCacheSizeLimit: (n: number) => void;
  readerFontSize: number;
  setReaderFontSize: (n: number) => void;
  cacheStatus: CacheStatusResponse | null;
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

      {/* Background refresh threshold */}
      <div className="flex flex-col gap-1.5">
        <label
          htmlFor="refresh-threshold"
          className="text-sm font-medium text-foreground"
        >
          Background Refresh Threshold (hours)
        </label>
        <input
          id="refresh-threshold"
          type="number"
          min={1}
          value={refreshThreshold}
          onChange={(e) => setRefreshThreshold(Number(e.target.value))}
          className="rounded-md border border-input bg-background px-3 py-2 text-sm focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
        />
        <p className="text-xs text-muted-foreground">
          How many hours the app must be open before a background refresh fires (default: 4)
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

      {/* Cache size limit */}
      <div className="flex flex-col gap-1.5">
        <label
          htmlFor="cache-size-limit"
          className="text-sm font-medium text-foreground"
        >
          Article Cache Size Limit (GB)
        </label>
        <input
          id="cache-size-limit"
          type="number"
          min={1}
          value={cacheSizeLimit}
          onChange={(e) => setCacheSizeLimit(Number(e.target.value))}
          className="rounded-md border border-input bg-background px-3 py-2 text-sm focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
        />
        <p className="text-xs text-muted-foreground">
          Maximum disk space for cached article bodies (default: 20)
        </p>
        {cacheStatus && (
          <div className="mt-2 flex flex-col gap-1">
            <div className="flex items-center justify-between text-xs text-muted-foreground">
              <span>
                {(cacheStatus.body_cache_size_bytes / 1_073_741_824).toFixed(2)} GB used
              </span>
              <span>{cacheStatus.size_cap_pct}%</span>
            </div>
            <div className="h-2 w-full rounded-full bg-muted">
              <div
                className={cn(
                  "h-2 rounded-full transition-all",
                  cacheStatus.size_cap_pct >= 90
                    ? "bg-destructive"
                    : cacheStatus.size_cap_pct >= 70
                      ? "bg-yellow-500"
                      : "bg-primary"
                )}
                style={{ width: `${Math.min(cacheStatus.size_cap_pct, 100)}%` }}
              />
            </div>
            <p className="text-xs text-muted-foreground">
              {cacheStatus.cached_articles} of {cacheStatus.total_articles} articles cached
            </p>
          </div>
        )}
      </div>

      {/* Reader font size */}
      <div className="flex flex-col gap-1.5">
        <label
          htmlFor="reader-font-size"
          className="text-sm font-medium text-foreground"
        >
          Reader Font Size (px)
        </label>
        <input
          id="reader-font-size"
          type="number"
          min={8}
          max={72}
          value={readerFontSize}
          onChange={(e) => setReaderFontSize(Number(e.target.value))}
          className="rounded-md border border-input bg-background px-3 py-2 text-sm focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
        />
        <p className="text-xs text-muted-foreground">
          Font size for article reader view (default: 16)
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
