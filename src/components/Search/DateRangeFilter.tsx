import { useState, useCallback } from "react";
import { Calendar, X } from "lucide-react";

interface DateRangeFilterProps {
  dateFrom: string | undefined;
  dateTo: string | undefined;
  onChange: (dateFrom: string | undefined, dateTo: string | undefined) => void;
}

/**
 * Date range filter with native date inputs for from/to filtering.
 * Uses HTML date inputs for reliable cross-platform support.
 */
export function DateRangeFilter({ dateFrom, dateTo, onChange }: DateRangeFilterProps) {
  const [isExpanded, setIsExpanded] = useState(
    Boolean(dateFrom || dateTo)
  );

  const handleClear = useCallback(() => {
    onChange(undefined, undefined);
    setIsExpanded(false);
  }, [onChange]);

  if (!isExpanded) {
    return (
      <button
        type="button"
        onClick={() => setIsExpanded(true)}
        className="inline-flex items-center gap-1.5 rounded-md border border-border px-3 py-1.5 text-xs font-medium text-muted-foreground hover:bg-accent hover:text-foreground transition-colors"
      >
        <Calendar className="h-3.5 w-3.5" />
        Date Range
      </button>
    );
  }

  return (
    <div className="flex items-center gap-2">
      <Calendar className="h-3.5 w-3.5 text-muted-foreground" />
      <div className="flex items-center gap-2">
        <label className="text-xs text-muted-foreground">From</label>
        <input
          type="date"
          value={dateFrom ?? ""}
          onChange={(e) => onChange(e.target.value || undefined, dateTo)}
          className="rounded-md border border-border bg-background px-2 py-1 text-xs text-foreground focus:outline-none focus:ring-1 focus:ring-ring"
        />
        <label className="text-xs text-muted-foreground">To</label>
        <input
          type="date"
          value={dateTo ?? ""}
          onChange={(e) => onChange(dateFrom, e.target.value || undefined)}
          className="rounded-md border border-border bg-background px-2 py-1 text-xs text-foreground focus:outline-none focus:ring-1 focus:ring-ring"
        />
      </div>
      <button
        type="button"
        onClick={handleClear}
        className="rounded-sm p-0.5 text-muted-foreground hover:text-foreground"
        title="Clear date filter"
      >
        <X className="h-3.5 w-3.5" />
      </button>
    </div>
  );
}
