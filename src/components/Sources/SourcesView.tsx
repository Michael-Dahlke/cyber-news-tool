import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { Loader2, Rss, Shield, Building2, Newspaper } from "lucide-react";
import * as feedService from "@/services/feedService";
import type { NewsSource } from "@/types";
import { cn } from "@/lib/utils";

const categoryConfig: Record<string, { label: string; icon: typeof Shield; color: string }> = {
  general: { label: "General", icon: Newspaper, color: "text-emerald-500" },
  microsoft: { label: "Microsoft", icon: Building2, color: "text-blue-500" },
  threat_intel: { label: "Threat Intel", icon: Shield, color: "text-red-500" },
};

/**
 * Grid view of all active news sources (FR-033).
 * Each card navigates to /sources/:id to list that source's articles.
 */
export function SourcesView() {
  const navigate = useNavigate();
  const [sources, setSources] = useState<NewsSource[]>([]);
  const [loading, setLoading] = useState(true);

  const fetchSources = useCallback(async () => {
    setLoading(true);
    try {
      const all = await feedService.listSources();
      setSources(all.filter((s) => s.is_active));
    } catch (err) {
      console.error("Failed to load sources:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchSources();
  }, [fetchSources]);

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (sources.length === 0) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-3 text-muted-foreground">
        <Rss className="h-12 w-12" />
        <p className="text-lg font-medium">No active sources</p>
        <p className="text-sm">Add sources in Settings to get started.</p>
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto">
      <div className="mx-auto max-w-5xl px-6 py-6">
        <h1 className="mb-6 text-xl font-bold text-foreground">Browse by Source</h1>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {sources.map((source) => {
            const cat = categoryConfig[source.default_tag] ?? categoryConfig.general!;
            const CatIcon = cat.icon;

            return (
              <button
                key={source.id}
                type="button"
                onClick={() => navigate(`/sources/${source.id}`)}
                className={cn(
                  "group flex flex-col gap-3 rounded-lg border border-border bg-card p-5 text-left transition-colors",
                  "hover:border-primary/30 hover:bg-accent/50"
                )}
              >
                <div className="flex items-center gap-2">
                  <Rss className="h-4 w-4 text-muted-foreground" />
                  <h2 className="text-sm font-semibold text-foreground group-hover:text-primary">
                    {source.name}
                  </h2>
                </div>

                <span
                  className={cn(
                    "inline-flex w-fit items-center gap-1 rounded-full px-2 py-0.5 text-xs font-medium",
                    cat.color
                  )}
                >
                  <CatIcon className="h-3 w-3" />
                  {cat.label}
                </span>

                <p className="truncate text-xs text-muted-foreground">{source.feed_url}</p>
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}
