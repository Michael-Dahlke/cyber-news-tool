import { useEffect, useState, useCallback } from "react";
import {
  Plus,
  Trash2,
  Power,
  PowerOff,
  Loader2,
  AlertCircle,
  CheckCircle2,
} from "lucide-react";
import { cn } from "@/lib/utils";
import type { NewsSource, ArticleCategory } from "@/types";
import * as feedService from "@/services/feedService";

export function SourceManager() {
  const [sources, setSources] = useState<NewsSource[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Add form state
  const [showAddForm, setShowAddForm] = useState(false);
  const [newUrl, setNewUrl] = useState("");
  const [newName, setNewName] = useState("");
  const [newTag, setNewTag] = useState<ArticleCategory>("general");
  const [adding, setAdding] = useState(false);
  const [addError, setAddError] = useState<string | null>(null);
  const [addSuccess, setAddSuccess] = useState(false);
  const [smokeTesting, setSmokeTesting] = useState(false);
  const [smokeSummary, setSmokeSummary] = useState<string | null>(null);

  const fetchSources = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await feedService.listSources();
      setSources(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchSources();
  }, [fetchSources]);

  const handleAddSource = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newUrl.trim()) return;

    setAdding(true);
    setAddError(null);
    setAddSuccess(false);

    try {
      await feedService.addSource({
        feed_url: newUrl.trim(),
        name: newName.trim() || undefined,
        default_tag: newTag,
      });
      setAddSuccess(true);
      setNewUrl("");
      setNewName("");
      setNewTag("general");
      setShowAddForm(false);
      await fetchSources();
      setTimeout(() => setAddSuccess(false), 3000);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      switch (msg) {
        case "INVALID_URL":
          setAddError("Invalid URL. Must start with http:// or https://");
          break;
        case "UNREACHABLE":
          setAddError("Could not reach the feed URL");
          break;
        case "UNPARSEABLE_FEED":
          setAddError("URL is not a valid RSS/Atom feed");
          break;
        case "DUPLICATE_URL":
          setAddError("This feed URL is already added");
          break;
        default:
          setAddError(msg);
      }
    } finally {
      setAdding(false);
    }
  };

  const handleSmokeTest = async () => {
    setSmokeTesting(true);
    setSmokeSummary(null);
    try {
      const results = await feedService.smokeTestFeeds();
      const liveCount = results.filter((r) => r.ok).length;
      const failed = results.filter((r) => !r.ok);

      if (failed.length === 0) {
        setSmokeSummary(`All ${liveCount} active feeds are responding successfully.`);
      } else {
        const failedNames = failed.map((r) => r.source_name).join(", ");
        setSmokeSummary(`${liveCount} active feeds are live; failing: ${failedNames}`);
      }
    } catch (err) {
      setSmokeSummary(err instanceof Error ? err.message : String(err));
    } finally {
      setSmokeTesting(false);
    }
  };

  const handleToggleActive = async (source: NewsSource) => {
    try {
      await feedService.updateSource({
        id: source.id,
        is_active: !source.is_active,
      });
      await fetchSources();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleRemove = async (source: NewsSource) => {
    if (source.is_default) return;

    try {
      await feedService.removeSource(source.id);
      await fetchSources();
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      if (msg === "CANNOT_REMOVE_DEFAULT") {
        setError("Default sources cannot be removed, only deactivated");
      } else {
        setError(msg);
      }
    }
  };

  if (loading) {
    return (
      <div className="flex items-center gap-2 py-4 text-sm text-muted-foreground">
        <Loader2 className="h-4 w-4 animate-spin" />
        Loading sources...
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h3 className="text-base font-semibold text-foreground">
          News Sources
        </h3>
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={handleSmokeTest}
            disabled={smokeTesting}
            className={cn(
              "inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 text-xs font-medium transition-colors",
              "bg-muted text-foreground hover:bg-muted/80",
              "disabled:pointer-events-none disabled:opacity-50"
            )}
          >
            {smokeTesting && <Loader2 className="h-3.5 w-3.5 animate-spin" />}
            {smokeTesting ? "Checking..." : "Check Feeds"}
          </button>
          <button
            type="button"
            onClick={() => setShowAddForm(!showAddForm)}
            className={cn(
              "inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 text-xs font-medium transition-colors",
              "bg-primary text-primary-foreground hover:bg-primary/90"
            )}
          >
            <Plus className="h-3.5 w-3.5" />
            Add Source
          </button>
        </div>
      </div>

      {smokeSummary && (
        <div className="rounded-md border border-border bg-card px-3 py-2 text-xs text-muted-foreground">
          {smokeSummary}
        </div>
      )}

      {/* Success message */}
      {addSuccess && (
        <div className="flex items-center gap-2 rounded-md border border-general/20 bg-general/5 px-3 py-2 text-sm text-general">
          <CheckCircle2 className="h-4 w-4" />
          Source added successfully
        </div>
      )}

      {/* Error display */}
      {error && (
        <div className="flex items-center gap-2 rounded-md border border-destructive/20 bg-destructive/5 px-3 py-2 text-sm text-destructive">
          <AlertCircle className="h-4 w-4" />
          {error}
        </div>
      )}

      {/* Add form */}
      {showAddForm && (
        <form
          onSubmit={handleAddSource}
          className="flex flex-col gap-3 rounded-lg border border-border bg-card p-4"
        >
          <div className="flex flex-col gap-1.5">
            <label
              htmlFor="feed-url"
              className="text-xs font-medium text-muted-foreground"
            >
              Feed URL *
            </label>
            <input
              id="feed-url"
              type="url"
              value={newUrl}
              onChange={(e) => setNewUrl(e.target.value)}
              placeholder="https://example.com/feed.xml"
              required
              className="rounded-md border border-input bg-background px-3 py-2 text-sm placeholder:text-muted-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
            />
          </div>

          <div className="flex flex-col gap-1.5">
            <label
              htmlFor="feed-name"
              className="text-xs font-medium text-muted-foreground"
            >
              Display Name (optional — auto-detected from feed)
            </label>
            <input
              id="feed-name"
              type="text"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              placeholder="My Feed"
              className="rounded-md border border-input bg-background px-3 py-2 text-sm placeholder:text-muted-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
            />
          </div>

          <div className="flex flex-col gap-1.5">
            <label
              htmlFor="feed-tag"
              className="text-xs font-medium text-muted-foreground"
            >
              Category
            </label>
            <select
              id="feed-tag"
              value={newTag}
              onChange={(e) => setNewTag(e.target.value as ArticleCategory)}
              className="rounded-md border border-input bg-background px-3 py-2 text-sm focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
            >
              <option value="general">General</option>
              <option value="microsoft">Microsoft</option>
              <option value="threat_intel">Threat Intelligence</option>
            </select>
          </div>

          {addError && (
            <div className="flex items-center gap-2 text-xs text-destructive">
              <AlertCircle className="h-3.5 w-3.5" />
              {addError}
            </div>
          )}

          <div className="flex items-center gap-2">
            <button
              type="submit"
              disabled={adding || !newUrl.trim()}
              className={cn(
                "inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 text-xs font-medium transition-colors",
                "bg-primary text-primary-foreground hover:bg-primary/90",
                "disabled:pointer-events-none disabled:opacity-50"
              )}
            >
              {adding && <Loader2 className="h-3.5 w-3.5 animate-spin" />}
              {adding ? "Validating..." : "Add Feed"}
            </button>
            <button
              type="button"
              onClick={() => {
                setShowAddForm(false);
                setAddError(null);
              }}
              className="rounded-md px-3 py-1.5 text-xs font-medium text-muted-foreground hover:text-foreground"
            >
              Cancel
            </button>
          </div>
        </form>
      )}

      {/* Sources list */}
      <div className="flex flex-col gap-1">
        {sources.map((source) => (
          <div
            key={source.id}
            className={cn(
              "flex items-center gap-3 rounded-lg border border-border px-3 py-2.5",
              !source.is_active && "opacity-50"
            )}
          >
            {/* Source info */}
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2">
                <span className="truncate text-sm font-medium text-foreground">
                  {source.name}
                </span>
                {source.is_default && (
                  <span className="shrink-0 rounded-full bg-muted px-2 py-0.5 text-[10px] font-medium text-muted-foreground">
                    Default
                  </span>
                )}
              </div>
              <p className="truncate text-xs text-muted-foreground">
                {source.feed_url}
              </p>
            </div>

            {/* Actions */}
            <div className="flex shrink-0 items-center gap-1">
              <button
                type="button"
                onClick={() => handleToggleActive(source)}
                title={source.is_active ? "Deactivate" : "Activate"}
                className={cn(
                  "rounded-md p-1.5 transition-colors",
                  source.is_active
                    ? "text-general hover:bg-general/10"
                    : "text-muted-foreground hover:bg-muted"
                )}
              >
                {source.is_active ? (
                  <Power className="h-4 w-4" />
                ) : (
                  <PowerOff className="h-4 w-4" />
                )}
              </button>

              {!source.is_default && (
                <button
                  type="button"
                  onClick={() => handleRemove(source)}
                  title="Remove source"
                  className="rounded-md p-1.5 text-muted-foreground transition-colors hover:bg-destructive/10 hover:text-destructive"
                >
                  <Trash2 className="h-4 w-4" />
                </button>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
