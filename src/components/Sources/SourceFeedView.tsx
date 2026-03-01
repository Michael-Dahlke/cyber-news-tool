import { useState, useEffect, useCallback } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { ArrowLeft, Loader2, Rss, AlertTriangle } from "lucide-react";
import { listBySource } from "@/services/articleService";
import type { ArticleSummary, SourceInfo } from "@/services/articleService";
import { ArticleCard } from "@/components/Dashboard/ArticleCard";

/**
 * Lists all articles for a single news source (FR-033).
 * Route: /sources/:sourceId
 */
export function SourceFeedView() {
  const { sourceId } = useParams<{ sourceId: string }>();
  const navigate = useNavigate();

  const [source, setSource] = useState<SourceInfo | null>(null);
  const [articles, setArticles] = useState<ArticleSummary[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [offset, setOffset] = useState(0);

  const PAGE_SIZE = 50;

  const fetchArticles = useCallback(
    async (currentOffset: number) => {
      if (!sourceId) return;
      setLoading(true);
      setError(null);
      try {
        const res = await listBySource({
          source_id: Number(sourceId),
          limit: PAGE_SIZE,
          offset: currentOffset,
        });
        setSource(res.source);
        setArticles(res.articles);
        setTotal(res.total);
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        if (msg === "SOURCE_NOT_FOUND") {
          setError("Source not found.");
        } else {
          setError("Failed to load articles.");
        }
      } finally {
        setLoading(false);
      }
    },
    [sourceId]
  );

  useEffect(() => {
    fetchArticles(offset);
  }, [fetchArticles, offset]);

  const handleArticleClick = useCallback(
    (article: ArticleSummary) => {
      navigate(`/article/${article.id}`);
    },
    [navigate]
  );

  if (loading && !source) {
    return (
      <div className="flex h-full items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-4">
        <AlertTriangle className="h-12 w-12 text-destructive" />
        <p className="text-lg font-medium">{error}</p>
        <button
          onClick={() => navigate("/sources")}
          className="inline-flex items-center gap-2 rounded-md bg-secondary px-4 py-2 text-sm font-medium text-secondary-foreground hover:bg-secondary/80"
        >
          <ArrowLeft className="h-4 w-4" />
          Back to Sources
        </button>
      </div>
    );
  }

  const hasMore = offset + PAGE_SIZE < total;
  const hasPrev = offset > 0;

  return (
    <div className="h-full overflow-y-auto">
      <div className="mx-auto max-w-5xl px-6 py-6">
        {/* Header */}
        <div className="mb-6 flex items-center gap-3">
          <button
            onClick={() => navigate("/sources")}
            className="inline-flex items-center gap-2 rounded-md px-3 py-1.5 text-sm font-medium text-muted-foreground hover:bg-accent hover:text-foreground"
          >
            <ArrowLeft className="h-4 w-4" />
            Sources
          </button>
          <div className="flex items-center gap-2">
            <Rss className="h-4 w-4 text-muted-foreground" />
            <h1 className="text-xl font-bold text-foreground">{source?.name ?? "Source"}</h1>
          </div>
          <span className="text-sm text-muted-foreground">
            {total} article{total !== 1 ? "s" : ""}
          </span>
        </div>

        {/* Article list */}
        {articles.length === 0 ? (
          <div className="flex flex-col items-center justify-center gap-3 py-12 text-muted-foreground">
            <Rss className="h-10 w-10" />
            <p className="text-sm">No articles from this source yet.</p>
          </div>
        ) : (
          <>
            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
              {articles.map((article) => (
                <ArticleCard key={article.id} article={article} onClick={handleArticleClick} />
              ))}
            </div>

            {/* Pagination */}
            {(hasPrev || hasMore) && (
              <div className="mt-6 flex items-center justify-center gap-4">
                <button
                  disabled={!hasPrev}
                  onClick={() => setOffset((o) => Math.max(0, o - PAGE_SIZE))}
                  className="rounded-md bg-secondary px-4 py-2 text-sm font-medium text-secondary-foreground hover:bg-secondary/80 disabled:opacity-40"
                >
                  Previous
                </button>
                <span className="text-xs text-muted-foreground">
                  {offset + 1}–{Math.min(offset + PAGE_SIZE, total)} of {total}
                </span>
                <button
                  disabled={!hasMore}
                  onClick={() => setOffset((o) => o + PAGE_SIZE)}
                  className="rounded-md bg-secondary px-4 py-2 text-sm font-medium text-secondary-foreground hover:bg-secondary/80 disabled:opacity-40"
                >
                  Next
                </button>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}
