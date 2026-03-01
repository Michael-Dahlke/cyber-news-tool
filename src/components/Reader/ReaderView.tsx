import { useState, useEffect, useCallback } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { getContent } from "@/services/articleService";
import type { ArticleDetail } from "@/services/articleService";
import { ContentRenderer } from "./ContentRenderer";
import { WebViewFallback } from "./WebViewFallback";
import { KQLPanel } from "@/components/KQL/KQLPanel";
import {
  ArrowLeft,
  ExternalLink,
  Clock,
  Loader2,
  AlertTriangle,
  Shield,
  Building2,
  Newspaper,
  ShieldAlert,
} from "lucide-react";

const categoryConfig: Record<string, { label: string; icon: typeof Shield; color: string }> = {
  general: { label: "General", icon: Newspaper, color: "text-emerald-500" },
  microsoft: { label: "Microsoft", icon: Building2, color: "text-blue-500" },
  threat_intel: { label: "Threat Intel", icon: Shield, color: "text-red-500" },
};

function formatDate(iso: string): string {
  try {
    return new Date(iso).toLocaleDateString("en-US", {
      weekday: "long",
      year: "numeric",
      month: "long",
      day: "numeric",
    });
  } catch {
    return iso;
  }
}

export function ReaderView() {
  const { articleId } = useParams<{ articleId: string }>();
  const navigate = useNavigate();

  const [article, setArticle] = useState<ArticleDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [kqlPanelOpen, setKqlPanelOpen] = useState(false);

  const fetchContent = useCallback(async () => {
    if (!articleId) return;

    setLoading(true);
    setError(null);

    try {
      const response = await getContent(Number(articleId));
      setArticle(response.article);
    } catch (err) {
      const errMsg = err instanceof Error ? err.message : String(err);
      if (errMsg === "NOT_FOUND") {
        setError("Article not found.");
      } else {
        setError("Failed to load article content.");
      }
    } finally {
      setLoading(false);
    }
  }, [articleId]);

  useEffect(() => {
    fetchContent();
  }, [fetchContent]);

  const handleBack = useCallback(() => {
    navigate(-1);
  }, [navigate]);

  const handleOpenInBrowser = useCallback(async () => {
    if (!article) return;
    // Use Tauri opener plugin to open in default browser
    try {
      const { openUrl } = await import("@tauri-apps/plugin-opener");
      await openUrl(article.url);
    } catch {
      // Fallback: window.open
      window.open(article.url, "_blank");
    }
  }, [article]);

  const handleWebViewExtractionComplete = useCallback(
    (updatedArticle: ArticleDetail) => {
      setArticle(updatedArticle);
    },
    []
  );

  // Loading state
  if (loading) {
    return (
      <div className="flex h-full items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  // Error state
  if (error || !article) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-4">
        <AlertTriangle className="h-12 w-12 text-destructive" />
        <p className="text-lg font-medium">{error ?? "Article not found"}</p>
        <button
          onClick={handleBack}
          className="inline-flex items-center gap-2 rounded-md bg-secondary px-4 py-2 text-sm font-medium text-secondary-foreground hover:bg-secondary/80"
        >
          <ArrowLeft className="h-4 w-4" />
          Go Back
        </button>
      </div>
    );
  }

  const catConfig = categoryConfig[article.category] ?? categoryConfig["general"]!;
  const CategoryIcon = catConfig.icon;

  // Determine content display mode
  const hasContent = article.body_html && article.body_html.trim().length > 0;
  const extractionFailed = article.extraction_method === "failed";

  return (
    <div className="flex h-full flex-col overflow-hidden">
      {/* Top bar with back button and actions */}
      <div className="flex items-center justify-between border-b border-border px-4 py-3">
        <button
          onClick={handleBack}
          className="inline-flex items-center gap-2 rounded-md px-3 py-1.5 text-sm font-medium text-muted-foreground hover:bg-accent hover:text-foreground"
        >
          <ArrowLeft className="h-4 w-4" />
          Back
        </button>

        <div className="flex items-center gap-2">
          <button
            onClick={() => setKqlPanelOpen(!kqlPanelOpen)}
            className={`inline-flex items-center gap-2 rounded-md px-3 py-1.5 text-sm font-medium transition-colors ${
              kqlPanelOpen
                ? "bg-primary/10 text-primary"
                : "text-muted-foreground hover:bg-accent hover:text-foreground"
            }`}
            title="Generate KQL Queries"
          >
            <ShieldAlert className="h-4 w-4" />
            KQL Queries
          </button>
          <button
            onClick={handleOpenInBrowser}
            className="inline-flex items-center gap-2 rounded-md px-3 py-1.5 text-sm font-medium text-muted-foreground hover:bg-accent hover:text-foreground"
            title="Open in browser"
          >
            <ExternalLink className="h-4 w-4" />
            Open in Browser
          </button>
        </div>
      </div>

      {/* Main content area with optional KQL side panel */}
      <div className="flex flex-1 overflow-hidden">
        {/* Scrollable article content */}
        <div className="flex-1 overflow-y-auto">
          <div className="mx-auto max-w-3xl px-6 py-8">
          {/* Article header */}
          <header className="mb-8">
            <div className="mb-3 flex items-center gap-2">
              <span
                className={`inline-flex items-center gap-1 text-xs font-medium ${catConfig.color}`}
              >
                <CategoryIcon className="h-3.5 w-3.5" />
                {catConfig.label}
              </span>
              <span className="text-xs text-muted-foreground">•</span>
              <span className="text-xs text-muted-foreground">
                {article.source_name}
              </span>
            </div>

            <h1 className="text-2xl font-bold leading-tight text-foreground mb-3">
              {article.title}
            </h1>

            <div className="flex items-center gap-4 text-sm text-muted-foreground">
              <span className="inline-flex items-center gap-1">
                <Clock className="h-3.5 w-3.5" />
                {formatDate(article.publish_date)}
              </span>
              {article.is_cached && (
                <span className="inline-flex items-center gap-1 text-xs text-emerald-600 dark:text-emerald-400">
                  Cached
                </span>
              )}
              {article.extraction_method && (
                <span className="text-xs text-muted-foreground/60">
                  via {article.extraction_method}
                </span>
              )}
            </div>
          </header>

          {/* Content area */}
          {hasContent ? (
            <ContentRenderer
              bodyHtml={article.body_html!}
              imageUrls={article.image_urls}
            />
          ) : extractionFailed ? (
            /* FR-015: Extraction failed fallback */
            <div className="flex flex-col items-center justify-center gap-4 py-12">
              <AlertTriangle className="h-12 w-12 text-amber-500" />
              <p className="text-lg font-medium text-foreground">
                Content could not be extracted
              </p>
              <p className="text-sm text-muted-foreground text-center max-w-md">
                We were unable to extract readable content from this article.
                You can view the original article in your browser.
              </p>
              <button
                onClick={handleOpenInBrowser}
                className="mt-2 inline-flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
              >
                <ExternalLink className="h-4 w-4" />
                Open in Browser
              </button>
            </div>
          ) : (
            /* WebView fallback extraction attempt */
            <WebViewFallback
              article={article}
              onExtractionComplete={handleWebViewExtractionComplete}
              onFallbackToBrowser={handleOpenInBrowser}
            />
          )}
        </div>
      </div>

        {/* KQL side panel */}
        {kqlPanelOpen && article && (
          <div className="w-[420px] flex-shrink-0 overflow-hidden">
            <KQLPanel
              articleId={article.id}
              articleTitle={article.title}
              onClose={() => setKqlPanelOpen(false)}
            />
          </div>
        )}
      </div>
    </div>
  );
}
