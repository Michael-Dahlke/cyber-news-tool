import { useState, useEffect, useCallback } from "react";
import { Copy, Check, Table2 } from "lucide-react";
import * as kqlService from "@/services/kqlService";
import type { KqlSuggestion } from "@/types";

interface KQLQueryCardProps {
  suggestion: KqlSuggestion;
}

export function KQLQueryCard({ suggestion }: KQLQueryCardProps) {
  const [highlightedHtml, setHighlightedHtml] = useState<string>("");
  const [copied, setCopied] = useState(false);

  // Syntax-highlight the query using shiki with Kusto grammar
  useEffect(() => {
    let cancelled = false;

    async function highlight() {
      try {
        const { codeToHtml } = await import("shiki");
        const html = await codeToHtml(suggestion.query_text, {
          lang: "kusto",
          theme: "github-dark",
        });
        if (!cancelled) {
          setHighlightedHtml(html);
        }
      } catch {
        // Fallback: plain text in a <pre> block
        if (!cancelled) {
          setHighlightedHtml("");
        }
      }
    }

    highlight();
    return () => {
      cancelled = true;
    };
  }, [suggestion.query_text]);

  const handleCopy = useCallback(async () => {
    try {
      await kqlService.copyToClipboard(suggestion.query_text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Fallback to browser clipboard API
      try {
        await navigator.clipboard.writeText(suggestion.query_text);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
      } catch {
        // Silently fail
      }
    }
  }, [suggestion.query_text]);

  return (
    <div className="rounded-lg border border-border bg-card overflow-hidden">
      {/* Header with description and table label */}
      <div className="flex items-start justify-between gap-3 px-4 py-3 border-b border-border bg-muted/30">
        <div className="flex-1 min-w-0">
          <p className="text-sm font-medium text-foreground leading-tight">
            {suggestion.description}
          </p>
          {suggestion.target_table && (
            <span className="mt-1 inline-flex items-center gap-1 text-xs text-muted-foreground">
              <Table2 className="h-3 w-3" />
              {suggestion.target_table}
            </span>
          )}
        </div>
        <button
          type="button"
          onClick={handleCopy}
          className="flex-shrink-0 inline-flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs font-medium text-muted-foreground hover:bg-accent hover:text-foreground transition-colors"
          title={copied ? "Copied!" : "Copy to clipboard"}
        >
          {copied ? (
            <>
              <Check className="h-3.5 w-3.5 text-emerald-500" />
              <span className="text-emerald-500">Copied</span>
            </>
          ) : (
            <>
              <Copy className="h-3.5 w-3.5" />
              Copy
            </>
          )}
        </button>
      </div>

      {/* Query code block */}
      <div className="relative overflow-x-auto">
        {highlightedHtml ? (
          <div
            className="kql-code-block text-sm [&_pre]:!m-0 [&_pre]:!rounded-none [&_pre]:!p-4 [&_code]:!text-xs"
            dangerouslySetInnerHTML={{ __html: highlightedHtml }}
          />
        ) : (
          <pre className="m-0 rounded-none bg-zinc-900 p-4 text-xs text-zinc-300 overflow-x-auto">
            <code>{suggestion.query_text}</code>
          </pre>
        )}
      </div>
    </div>
  );
}
