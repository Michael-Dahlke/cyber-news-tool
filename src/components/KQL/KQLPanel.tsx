import { useState, useEffect, useCallback } from "react";
import {
  Loader2,
  AlertTriangle,
  RefreshCw,
  ShieldAlert,
  Settings2,
  X,
} from "lucide-react";
import { useNavigate } from "react-router-dom";
import * as kqlService from "@/services/kqlService";
import type { KqlSuggestion } from "@/types";
import { KQLQueryCard } from "./KQLQueryCard";

interface KQLPanelProps {
  articleId: number;
  articleTitle: string;
  onClose: () => void;
}

type PanelState =
  | "loading-existing"
  | "idle"
  | "confirming"
  | "generating"
  | "error"
  | "not-configured"
  | "done";

export function KQLPanel({ articleId, articleTitle, onClose }: KQLPanelProps) {
  const navigate = useNavigate();

  const [state, setState] = useState<PanelState>("loading-existing");
  const [suggestions, setSuggestions] = useState<KqlSuggestion[]>([]);
  const [generatedAt, setGeneratedAt] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [endpointDisplay, setEndpointDisplay] = useState<string>("");

  // On mount: check for existing suggestions
  useEffect(() => {
    let cancelled = false;

    async function loadExisting() {
      try {
        const result = await kqlService.getForArticle(articleId);
        if (cancelled) return;

        if (result.suggestions.length > 0) {
          setSuggestions(result.suggestions);
          setGeneratedAt(result.generated_at);
          setState("done");
        } else {
          setState("idle");
        }
      } catch {
        if (!cancelled) {
          setState("idle");
        }
      }
    }

    loadExisting();
    return () => {
      cancelled = true;
    };
  }, [articleId]);

  // Check LLM config and show confirmation dialog
  const handleGenerateClick = useCallback(async () => {
    setError(null);
    try {
      const config = await kqlService.checkConfig();
      if (!config.is_configured) {
        setState("not-configured");
        return;
      }
      setEndpointDisplay(config.endpoint);
      setState("confirming");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setState("error");
    }
  }, []);

  // User confirms → actually generate
  const handleConfirm = useCallback(async () => {
    setState("generating");
    setError(null);

    try {
      const result = await kqlService.generate(articleId, true);
      setSuggestions(result.suggestions);
      setEndpointDisplay(result.llm_endpoint);
      setGeneratedAt(new Date().toISOString());
      setState("done");
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      if (msg === "NOT_CONFIGURED" || msg.includes("NOT_CONFIGURED")) {
        setState("not-configured");
      } else if (msg === "LLM_NO_SUGGESTIONS" || msg.includes("LLM_NO_SUGGESTIONS")) {
        setSuggestions([]);
        setState("done");
        setError("No detectable threat indicators found in this article.");
      } else {
        setError(msg);
        setState("error");
      }
    }
  }, [articleId]);

  const handleCancel = useCallback(() => {
    setState("idle");
  }, []);

  const handleGoToSettings = useCallback(() => {
    navigate("/settings");
  }, [navigate]);

  return (
    <div className="flex h-full flex-col border-l border-border bg-background">
      {/* Panel header */}
      <div className="flex items-center justify-between border-b border-border px-4 py-3">
        <div className="flex items-center gap-2">
          <ShieldAlert className="h-4 w-4 text-primary" />
          <h2 className="text-sm font-semibold text-foreground">
            KQL Queries
          </h2>
        </div>
        <button
          type="button"
          onClick={onClose}
          className="rounded-md p-1 text-muted-foreground hover:bg-accent hover:text-foreground"
          title="Close panel"
        >
          <X className="h-4 w-4" />
        </button>
      </div>

      {/* Panel body */}
      <div className="flex-1 overflow-y-auto p-4">
        {/* Loading existing suggestions */}
        {state === "loading-existing" && (
          <div className="flex flex-col items-center justify-center gap-3 py-8">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
            <p className="text-sm text-muted-foreground">
              Checking for existing queries...
            </p>
          </div>
        )}

        {/* Idle: no suggestions yet */}
        {state === "idle" && (
          <div className="flex flex-col items-center justify-center gap-4 py-8">
            <ShieldAlert className="h-10 w-10 text-muted-foreground/50" />
            <div className="text-center">
              <p className="text-sm font-medium text-foreground">
                Generate KQL Detection Queries
              </p>
              <p className="mt-1 text-xs text-muted-foreground max-w-[280px]">
                Use AI to generate Microsoft Sentinel / Defender XDR queries
                based on this article&apos;s threat context.
              </p>
            </div>
            <button
              type="button"
              onClick={handleGenerateClick}
              className="inline-flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
            >
              Generate KQL Queries
            </button>
          </div>
        )}

        {/* Confirmation dialog — Constitution II compliance */}
        {state === "confirming" && (
          <div className="flex flex-col gap-4 py-4">
            <div className="rounded-lg border border-amber-500/30 bg-amber-500/5 p-4">
              <div className="flex items-start gap-3">
                <AlertTriangle className="h-5 w-5 flex-shrink-0 text-amber-500 mt-0.5" />
                <div className="flex flex-col gap-2">
                  <p className="text-sm font-semibold text-foreground">
                    AI Query Generation
                  </p>
                  <p className="text-xs text-muted-foreground leading-relaxed">
                    This will send the article content to an external AI service
                    to generate detection queries. Please confirm you want to
                    proceed.
                  </p>
                  <dl className="mt-2 grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-xs">
                    <dt className="text-muted-foreground">Endpoint:</dt>
                    <dd className="font-mono text-foreground truncate">
                      {endpointDisplay}
                    </dd>
                    <dt className="text-muted-foreground">Article:</dt>
                    <dd className="text-foreground line-clamp-2">
                      {articleTitle}
                    </dd>
                  </dl>
                </div>
              </div>
            </div>

            <div className="flex gap-2">
              <button
                type="button"
                onClick={handleConfirm}
                className="flex-1 inline-flex items-center justify-center gap-2 rounded-md bg-primary px-3 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
              >
                Confirm & Generate
              </button>
              <button
                type="button"
                onClick={handleCancel}
                className="flex-1 inline-flex items-center justify-center gap-2 rounded-md border border-border px-3 py-2 text-sm font-medium text-muted-foreground hover:bg-accent hover:text-foreground"
              >
                Cancel
              </button>
            </div>
          </div>
        )}

        {/* Generating (loading) */}
        {state === "generating" && (
          <div className="flex flex-col items-center justify-center gap-3 py-8">
            <Loader2 className="h-6 w-6 animate-spin text-primary" />
            <p className="text-sm text-muted-foreground">
              Generating KQL queries...
            </p>
            <p className="text-xs text-muted-foreground/60">
              This may take up to 60 seconds
            </p>
          </div>
        )}

        {/* Not configured */}
        {state === "not-configured" && (
          <div className="flex flex-col items-center justify-center gap-4 py-8">
            <Settings2 className="h-10 w-10 text-muted-foreground/50" />
            <div className="text-center">
              <p className="text-sm font-medium text-foreground">
                LLM Not Configured
              </p>
              <p className="mt-1 text-xs text-muted-foreground max-w-[280px]">
                Configure an OpenAI-compatible API endpoint and API key in
                Settings to generate KQL queries.
              </p>
            </div>
            <button
              type="button"
              onClick={handleGoToSettings}
              className="inline-flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
            >
              <Settings2 className="h-4 w-4" />
              Go to Settings
            </button>
          </div>
        )}

        {/* Error state */}
        {state === "error" && (
          <div className="flex flex-col items-center justify-center gap-4 py-8">
            <AlertTriangle className="h-10 w-10 text-destructive" />
            <div className="text-center">
              <p className="text-sm font-medium text-foreground">
                Generation Failed
              </p>
              <p className="mt-1 text-xs text-muted-foreground max-w-[280px]">
                {error ?? "An unexpected error occurred."}
              </p>
            </div>
            <button
              type="button"
              onClick={handleGenerateClick}
              className="inline-flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
            >
              <RefreshCw className="h-4 w-4" />
              Retry
            </button>
          </div>
        )}

        {/* Results */}
        {state === "done" && (
          <div className="flex flex-col gap-4">
            {/* Generation metadata */}
            {generatedAt && (
              <div className="flex items-center justify-between text-xs text-muted-foreground">
                <span>
                  Generated{" "}
                  {new Date(generatedAt).toLocaleDateString("en-US", {
                    month: "short",
                    day: "numeric",
                    hour: "numeric",
                    minute: "2-digit",
                  })}
                </span>
                <button
                  type="button"
                  onClick={handleGenerateClick}
                  className="inline-flex items-center gap-1 hover:text-foreground"
                  title="Regenerate"
                >
                  <RefreshCw className="h-3 w-3" />
                  Regenerate
                </button>
              </div>
            )}

            {/* No suggestions message */}
            {suggestions.length === 0 && error && (
              <div className="rounded-lg border border-border bg-muted/30 p-4 text-center">
                <p className="text-sm text-muted-foreground">{error}</p>
              </div>
            )}

            {suggestions.length === 0 && !error && (
              <div className="rounded-lg border border-border bg-muted/30 p-4 text-center">
                <p className="text-sm text-muted-foreground">
                  No KQL suggestions were generated for this article.
                </p>
              </div>
            )}

            {/* Suggestion cards */}
            {suggestions.map((suggestion) => (
              <KQLQueryCard key={suggestion.id} suggestion={suggestion} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
