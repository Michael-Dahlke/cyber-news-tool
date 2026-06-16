import { useNavigate } from "react-router-dom";
import { NewsSection } from "./NewsSection";
import type { ArticleSummary } from "@/services/articleService";

const MSRC_RESTORE_KEY = "msrc:restore";

function getScrollContainer(): HTMLElement | null {
  return document.getElementById("app-scroll-container");
}

/**
 * Dedicated view for the Microsoft Security Response Center (MSRC) feed.
 * MSRC publishes a high volume of CVE advisories, so it lives on its own tab
 * to keep the main dashboard uncluttered.
 */
export function MSRCView() {
  const navigate = useNavigate();

  const handleArticleClick = (article: ArticleSummary) => {
    const container = getScrollContainer();
    sessionStorage.setItem(
      MSRC_RESTORE_KEY,
      JSON.stringify({ articleId: article.id, scrollTop: container?.scrollTop ?? 0 })
    );
    navigate(`/article/${article.id}`);
  };

  return (
    <div className="flex flex-col gap-6 p-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-foreground">MSRC / CVEs</h1>
        <p className="text-sm text-muted-foreground">
          Security advisories and CVEs from the Microsoft Security Response Center
        </p>
      </div>

      {/* MSRC articles: last 30 days, including today */}
      <NewsSection
        title="Recent advisories"
        category="msrc"
        daysBack={30}
        limit={200}
        onArticleClick={handleArticleClick}
      />
    </div>
  );
}
