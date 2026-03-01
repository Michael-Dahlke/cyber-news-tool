import { useState, useRef, useCallback } from "react";
import { extractViaWebview } from "@/services/articleService";
import type { ArticleDetail } from "@/services/articleService";
import { Loader2, AlertTriangle, ExternalLink } from "lucide-react";

interface WebViewFallbackProps {
  article: ArticleDetail;
  onExtractionComplete: (article: ArticleDetail) => void;
  onFallbackToBrowser: () => void;
}

/**
 * Loads the article URL in a hidden iframe and attempts to extract
 * readable content using a postMessage-based extraction approach.
 * If extraction fails or times out, offers "Open in browser" fallback.
 */
export function WebViewFallback({
  article,
  onExtractionComplete,
  onFallbackToBrowser,
}: WebViewFallbackProps) {
  const [status, setStatus] = useState<"loading" | "extracting" | "failed">(
    "loading"
  );
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleIframeLoad = useCallback(() => {
    setStatus("extracting");

    // Try to extract content from iframe
    try {
      const iframe = iframeRef.current;
      if (!iframe?.contentDocument) {
        // Cross-origin — can't access content, fall back
        setStatus("failed");
        return;
      }

      const doc = iframe.contentDocument;

      // Try to get the main article content
      const selectors = [
        "article",
        "[role='main']",
        ".post-content",
        ".entry-content",
        ".article-body",
        "main",
      ];

      let extractedHtml = "";
      for (const selector of selectors) {
        const el = doc.querySelector(selector);
        if (el && el.innerHTML.trim().length > 100) {
          extractedHtml = el.innerHTML;
          break;
        }
      }

      // If we couldn't find article content, try body
      if (!extractedHtml && doc.body) {
        // Remove nav, header, footer, aside, script, style elements
        const clone = doc.body.cloneNode(true) as HTMLElement;
        const removeSelectors = [
          "nav",
          "header",
          "footer",
          "aside",
          "script",
          "style",
          ".sidebar",
          ".navigation",
          ".comments",
          ".ad",
          ".advertisement",
        ];
        for (const sel of removeSelectors) {
          clone.querySelectorAll(sel).forEach((el) => el.remove());
        }
        extractedHtml = clone.innerHTML;
      }

      if (extractedHtml.trim().length > 100) {
        // Send extracted content to backend
        extractViaWebview(article.id, extractedHtml)
          .then((response) => {
            onExtractionComplete(response.article);
          })
          .catch(() => {
            setStatus("failed");
          });
      } else {
        setStatus("failed");
      }
    } catch {
      // Cross-origin or other error
      setStatus("failed");
    }
  }, [article.id, onExtractionComplete]);

  // Set a timeout for iframe loading
  const handleIframeRef = useCallback(
    (el: HTMLIFrameElement | null) => {
      (iframeRef as React.MutableRefObject<HTMLIFrameElement | null>).current = el;
      if (el) {
        // Set 15s timeout for iframe loading
        if (timeoutRef.current) clearTimeout(timeoutRef.current);
        timeoutRef.current = setTimeout(() => {
          if (status === "loading") {
            setStatus("failed");
          }
        }, 15000);
      }
    },
    [status]
  );

  if (status === "failed") {
    return (
      <div className="flex flex-col items-center justify-center gap-4 py-12">
        <AlertTriangle className="h-12 w-12 text-amber-500" />
        <p className="text-lg font-medium text-foreground">
          Content extraction failed
        </p>
        <p className="text-sm text-muted-foreground text-center max-w-md">
          We couldn&apos;t extract readable content from this article. You can
          view it directly in your browser.
        </p>
        <button
          onClick={onFallbackToBrowser}
          className="mt-2 inline-flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
        >
          <ExternalLink className="h-4 w-4" />
          Open in Browser
        </button>
      </div>
    );
  }

  return (
    <div className="relative">
      {/* Loading indicator */}
      <div className="flex items-center justify-center gap-2 py-8">
        <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
        <span className="text-sm text-muted-foreground">
          {status === "loading"
            ? "Loading article..."
            : "Extracting content..."}
        </span>
      </div>

      {/* Hidden iframe for content extraction */}
      <iframe
        ref={handleIframeRef}
        src={article.url}
        onLoad={handleIframeLoad}
        sandbox="allow-same-origin"
        className="absolute h-0 w-0 overflow-hidden opacity-0"
        title="Article content extraction"
      />
    </div>
  );
}
