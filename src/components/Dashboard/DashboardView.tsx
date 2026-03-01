import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { RefreshCw, Loader2 } from "lucide-react";
import { listen } from "@tauri-apps/api/event";
import { NewsSection } from "./NewsSection";
import { OfflineIndicator } from "@/components/Layout/OfflineIndicator";
import * as feedService from "@/services/feedService";
import { cn } from "@/lib/utils";
import type { ArticleSummary } from "@/services/articleService";

/**
 * Module-level flag so we only trigger one auto-refresh per app session,
 * regardless of how many times DashboardView mounts (e.g. navigating away and back).
 */
let sessionRefreshDone = false;

export function DashboardView() {
  const navigate = useNavigate();
  const [lastUpdated, setLastUpdated] = useState<string | null>(null);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [refreshKey, setRefreshKey] = useState(0);
  const [refreshProgress, setRefreshProgress] = useState<{
    current: number;
    total: number;
    source_name: string;
  } | null>(null);

  // Manual refresh handler
  const handleRefresh = useCallback(async () => {
    setIsRefreshing(true);
    try {
      await feedService.refreshFeeds();
      setLastUpdated(new Date().toISOString());
      setRefreshKey((k) => k + 1); // Force sections to re-fetch
    } catch (err) {
      console.error("Feed refresh failed:", err);
    } finally {
      setIsRefreshing(false);
      setRefreshProgress(null);
    }
  }, []);

  // Listen for auto-refresh events and progress
  useEffect(() => {
    const unlistenProgress = listen<{
      current: number;
      total: number;
      source_name: string;
    }>("feeds://refresh-progress", (event) => {
      setRefreshProgress(event.payload);
    });

    const unlistenComplete = listen<{
      articles_added: number;
      timestamp: string;
    }>("feeds://auto-refresh-complete", (event) => {
      setLastUpdated(event.payload.timestamp);
      setRefreshKey((k) => k + 1);
      setRefreshProgress(null);
    });

    return () => {
      unlistenProgress.then((fn) => fn());
      unlistenComplete.then((fn) => fn());
    };
  }, []);

  // Initial refresh on mount — only once per app session, not on every navigation.
  useEffect(() => {
    if (!sessionRefreshDone) {
      sessionRefreshDone = true;
      handleRefresh();
    }
  }, []);

  const handleArticleClick = (article: ArticleSummary) => {
    navigate(`/article/${article.id}`);
  };

  return (
    <div className="flex flex-col gap-6 p-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-foreground">Dashboard</h1>
          <p className="text-sm text-muted-foreground">
            Cybersecurity news from your configured sources
          </p>
        </div>
        <button
          type="button"
          onClick={handleRefresh}
          disabled={isRefreshing}
          className={cn(
            "inline-flex items-center gap-2 rounded-md px-4 py-2 text-sm font-medium transition-colors",
            "bg-primary text-primary-foreground hover:bg-primary/90",
            "disabled:pointer-events-none disabled:opacity-50"
          )}
        >
          {isRefreshing ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <RefreshCw className="h-4 w-4" />
          )}
          {isRefreshing ? "Refreshing..." : "Refresh Feeds"}
        </button>
      </div>

      {/* Offline indicator */}
      <OfflineIndicator />

      {/* Refresh progress */}
      {refreshProgress && isRefreshing && (
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <Loader2 className="h-3.5 w-3.5 animate-spin" />
          Fetching {refreshProgress.source_name} ({refreshProgress.current}/
          {refreshProgress.total})
        </div>
      )}

      {/* Three dashboard sections */}
      <div className="grid gap-8" key={refreshKey}>
        {/* Today's News: ALL categories published today (FR-002) */}
        <NewsSection
          title="Today"
          todayOnly
          lastUpdated={lastUpdated}
          onArticleClick={handleArticleClick}
        />

        {/* Microsoft News: last 7 days, excluding today (FR-003) */}
        <NewsSection
          title="Microsoft"
          category="microsoft"
          daysBack={7}
          excludeToday
          lastUpdated={lastUpdated}
          onArticleClick={handleArticleClick}
        />

        {/* Threat Intelligence: last 7 days, excluding today (FR-004) */}
        <NewsSection
          title="Threat Intelligence"
          category="threat_intel"
          daysBack={7}
          excludeToday
          lastUpdated={lastUpdated}
          onArticleClick={handleArticleClick}
        />

        {/* General: last 7 days, excluding today (FR-005) */}
        <NewsSection
          title="General"
          category="general"
          daysBack={7}
          excludeToday
          lastUpdated={lastUpdated}
          onArticleClick={handleArticleClick}
        />
      </div>
    </div>
  );
}
