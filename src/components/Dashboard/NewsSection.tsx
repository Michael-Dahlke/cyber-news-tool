import { useState, useEffect, useCallback } from "react";
import { ArticleCard } from "./ArticleCard";
import { listByCategory, listToday } from "@/services/articleService";
import type { ArticleSummary } from "@/services/articleService";
import type { ArticleCategory } from "@/types";
import { RefreshCw, Loader2, Inbox } from "lucide-react";
import { cn } from "@/lib/utils";

interface NewsSectionProps {
  title: string;
  /** Required unless todayOnly is true */
  category?: ArticleCategory;
  daysBack?: number;
  /** When true, fetches ALL articles from today regardless of category */
  todayOnly?: boolean;
  /** When true, excludes today's articles from the category query */
  excludeToday?: boolean;
  /** Max number of articles to fetch (defaults to 50 for categories). */
  limit?: number;
  lastUpdated?: string | null;
  onArticleClick?: (article: ArticleSummary) => void;
}

export function NewsSection({
  title,
  category,
  daysBack = 7,
  todayOnly = false,
  excludeToday = false,
  limit = 50,
  lastUpdated,
  onArticleClick,
}: NewsSectionProps) {
  const [articles, setArticles] = useState<ArticleSummary[]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchArticles = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      if (todayOnly) {
        const response = await listToday({ limit: 100 });
        setArticles(response.articles);
        setTotalCount(response.total_count);
      } else if (category) {
        const response = await listByCategory({
          category,
          days_back: daysBack,
          limit,
          exclude_today: excludeToday,
        });
        setArticles(response.articles);
        setTotalCount(response.total_count);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsLoading(false);
    }
  }, [category, daysBack, todayOnly, excludeToday, limit]);

  useEffect(() => {
    fetchArticles();
  }, [fetchArticles]);

  const formattedLastUpdated = lastUpdated
    ? new Date(lastUpdated).toLocaleTimeString([], {
        hour: "2-digit",
        minute: "2-digit",
      })
    : null;

  return (
    <section className="flex flex-col gap-3">
      {/* Section header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <h2 className="text-lg font-semibold text-foreground">{title}</h2>
          {totalCount > 0 && (
            <span className="rounded-full bg-muted px-2 py-0.5 text-xs font-medium text-muted-foreground">
              {totalCount}
            </span>
          )}
        </div>
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          {formattedLastUpdated && (
            <span>Updated {formattedLastUpdated}</span>
          )}
          <button
            type="button"
            onClick={fetchArticles}
            disabled={isLoading}
            className="rounded-md p-1 transition-colors hover:bg-accent"
            title="Refresh section"
          >
            <RefreshCw
              className={cn("h-3.5 w-3.5", isLoading && "animate-spin")}
            />
          </button>
        </div>
      </div>

      {/* Content */}
      {isLoading && articles.length === 0 ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
        </div>
      ) : error ? (
        <div className="rounded-lg border border-destructive/30 bg-destructive/5 p-4 text-sm text-destructive">
          Failed to load articles: {error}
        </div>
      ) : articles.length === 0 ? (
        <div className="flex flex-col items-center justify-center gap-2 rounded-lg border border-dashed border-border py-12 text-muted-foreground">
          <Inbox className="h-8 w-8" />
          <p className="text-sm">No articles found</p>
          <p className="text-xs">
            Articles will appear here after the next feed refresh.
          </p>
        </div>
      ) : (
        <div className="grid gap-2">
          {articles.map((article) => (
            <ArticleCard
              key={article.id}
              article={article}
              onClick={onArticleClick}
            />
          ))}
        </div>
      )}
    </section>
  );
}
