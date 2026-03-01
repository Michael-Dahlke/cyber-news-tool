import { cn } from "@/lib/utils";
import type { ArticleSummary } from "@/services/articleService";
import { ExternalLink, Clock, Shield, Building2, Newspaper } from "lucide-react";

const categoryConfig = {
  general: {
    label: "General",
    className: "bg-general/10 text-general",
    icon: Newspaper,
  },
  microsoft: {
    label: "Microsoft",
    className: "bg-microsoft/10 text-microsoft",
    icon: Building2,
  },
  threat_intel: {
    label: "Threat Intel",
    className: "bg-threat/10 text-threat",
    icon: Shield,
  },
} as const;

function formatRelativeDate(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMin = Math.floor(diffMs / 60_000);
  const diffHrs = Math.floor(diffMs / 3_600_000);
  const diffDays = Math.floor(diffMs / 86_400_000);

  if (diffMin < 1) return "Just now";
  if (diffMin < 60) return `${diffMin}m ago`;
  if (diffHrs < 24) return `${diffHrs}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;
  return date.toLocaleDateString();
}

interface ArticleCardProps {
  article: ArticleSummary;
  onClick?: (article: ArticleSummary) => void;
}

export function ArticleCard({ article, onClick }: ArticleCardProps) {
  const config = categoryConfig[article.category as keyof typeof categoryConfig] ?? categoryConfig.general;
  const CategoryIcon = config.icon;

  return (
    <button
      type="button"
      onClick={() => onClick?.(article)}
      className={cn(
        "group flex w-full flex-col gap-2 rounded-lg border border-border bg-card p-4 text-left transition-colors",
        "hover:border-primary/30 hover:bg-accent/50"
      )}
    >
      {/* Header: Category badge + time */}
      <div className="flex items-center justify-between gap-2">
        <span
          className={cn(
            "inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-medium",
            config.className
          )}
        >
          <CategoryIcon className="h-3 w-3" />
          {config.label}
        </span>
        <span className="flex items-center gap-1 text-xs text-muted-foreground">
          <Clock className="h-3 w-3" />
          {formatRelativeDate(article.publish_date)}
        </span>
      </div>

      {/* Title */}
      <h3 className="text-sm font-medium leading-snug text-foreground group-hover:text-primary">
        {article.title}
      </h3>

      {/* Footer: Source + external link icon */}
      <div className="flex items-center justify-between text-xs text-muted-foreground">
        <span>{article.source_name}</span>
        <ExternalLink className="h-3 w-3 opacity-0 transition-opacity group-hover:opacity-100" />
      </div>
    </button>
  );
}
