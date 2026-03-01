import { useState, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { SearchBar } from "./SearchBar";
import { DateRangeFilter } from "./DateRangeFilter";
import { searchQuery } from "@/services/searchService";
import type { SearchResult, SearchQueryResponse } from "@/services/searchService";
import { Search, Clock, Loader2, FileQuestion } from "lucide-react";
import { cn } from "@/lib/utils";

function formatDate(dateStr: string): string {
  try {
    return new Date(dateStr).toLocaleDateString("en-US", {
      month: "short",
      day: "numeric",
      year: "numeric",
    });
  } catch {
    return dateStr;
  }
}

export function SearchResults() {
  const navigate = useNavigate();

  const [response, setResponse] = useState<SearchQueryResponse | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [lastQuery, setLastQuery] = useState<string>("");
  const [dateFrom, setDateFrom] = useState<string | undefined>();
  const [dateTo, setDateTo] = useState<string | undefined>();

  const handleSearch = useCallback(
    async (keywords: string) => {
      setIsLoading(true);
      setError(null);
      setLastQuery(keywords);

      try {
        const result = await searchQuery({
          keywords,
          date_from: dateFrom,
          date_to: dateTo,
        });
        setResponse(result);
      } catch (err) {
        const errMsg = err instanceof Error ? err.message : String(err);
        if (errMsg === "INVALID_QUERY") {
          setError("Invalid search syntax. Check your query for unbalanced quotes.");
        } else if (errMsg === "QUERY_TOO_SHORT") {
          setError("Please enter a search term.");
        } else {
          setError("Search failed. Please try again.");
        }
        setResponse(null);
      } finally {
        setIsLoading(false);
      }
    },
    [dateFrom, dateTo]
  );

  const handleDateChange = useCallback(
    (from: string | undefined, to: string | undefined) => {
      setDateFrom(from);
      setDateTo(to);
      // Re-run search if we have a query
      if (lastQuery) {
        handleSearch(lastQuery);
      }
    },
    [lastQuery, handleSearch]
  );

  const handleResultClick = useCallback(
    (result: SearchResult) => {
      navigate(`/article/${result.article_id}`);
    },
    [navigate]
  );

  return (
    <div className="flex flex-col gap-6 p-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-foreground">Search</h1>
        <p className="text-sm text-muted-foreground">
          Full-text search across all articles
        </p>
      </div>

      {/* Search bar */}
      <SearchBar onSearch={handleSearch} isLoading={isLoading} />

      {/* Filters */}
      <div className="flex items-center gap-4">
        <DateRangeFilter
          dateFrom={dateFrom}
          dateTo={dateTo}
          onChange={handleDateChange}
        />
        {response && (
          <span className="text-xs text-muted-foreground">
            {response.total_count} result{response.total_count !== 1 ? "s" : ""}{" "}
            in {response.query_time_ms}ms
          </span>
        )}
      </div>

      {/* Error state */}
      {error && (
        <div className="rounded-lg border border-destructive/50 bg-destructive/10 p-4">
          <p className="text-sm text-destructive">{error}</p>
        </div>
      )}

      {/* Loading state */}
      {isLoading && (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
        </div>
      )}

      {/* Results */}
      {!isLoading && response && (
        <div className="flex flex-col gap-3">
          {response.results.length === 0 ? (
            <div className="flex flex-col items-center justify-center gap-3 py-12">
              <FileQuestion className="h-10 w-10 text-muted-foreground" />
              <p className="text-sm text-muted-foreground">
                No results found for &quot;{lastQuery}&quot;
              </p>
            </div>
          ) : (
            response.results.map((result) => (
              <button
                key={result.article_id}
                type="button"
                onClick={() => handleResultClick(result)}
                className={cn(
                  "flex w-full flex-col gap-2 rounded-lg border border-border bg-card p-4 text-left transition-colors",
                  "hover:border-primary/30 hover:bg-accent/50"
                )}
              >
                {/* Title */}
                <h3 className="text-sm font-medium leading-snug text-foreground hover:text-primary">
                  {result.title}
                </h3>

                {/* Excerpt with highlights */}
                <p
                  className="text-xs leading-relaxed text-muted-foreground [&>mark]:bg-yellow-200 [&>mark]:text-foreground dark:[&>mark]:bg-yellow-800"
                  dangerouslySetInnerHTML={{ __html: result.excerpt }}
                />

                {/* Footer: source + date */}
                <div className="flex items-center gap-3 text-xs text-muted-foreground">
                  <span className="inline-flex items-center gap-1">
                    <Search className="h-3 w-3" />
                    {result.source_name}
                  </span>
                  <span className="inline-flex items-center gap-1">
                    <Clock className="h-3 w-3" />
                    {formatDate(result.publish_date)}
                  </span>
                </div>
              </button>
            ))
          )}
        </div>
      )}

      {/* Empty state - no search yet */}
      {!isLoading && !response && !error && (
        <div className="flex flex-col items-center justify-center gap-3 py-12">
          <Search className="h-10 w-10 text-muted-foreground" />
          <p className="text-sm text-muted-foreground">
            Enter keywords to search across all articles
          </p>
        </div>
      )}
    </div>
  );
}
