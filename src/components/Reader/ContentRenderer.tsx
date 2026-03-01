import { useRef, useEffect } from "react";

interface ContentRendererProps {
  bodyHtml: string;
  imageUrls: string[];
}

/**
 * Renders cleaned article HTML with inline images and typography styles.
 * Uses dangerouslySetInnerHTML for the extracted content, with a prose
 * container for consistent typography.
 */
export function ContentRenderer({ bodyHtml, imageUrls }: ContentRendererProps) {
  const contentRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!contentRef.current) return;

    // Ensure all images have proper loading and alt attributes
    const images = contentRef.current.querySelectorAll("img");
    images.forEach((img) => {
      img.setAttribute("loading", "lazy");
      if (!img.alt) {
        img.alt = "Article image";
      }
      // Add responsive styling
      img.classList.add("max-w-full", "h-auto", "rounded-lg", "my-4");
    });

    // Make links open externally
    const links = contentRef.current.querySelectorAll("a");
    links.forEach((link) => {
      link.setAttribute("target", "_blank");
      link.setAttribute("rel", "noopener noreferrer");
      link.classList.add("text-primary", "underline", "hover:text-primary/80");
    });
  }, [bodyHtml]);

  return (
    <div className="mx-auto max-w-3xl">
      {/* Image gallery if article has notable images */}
      {imageUrls.length > 0 && (
        <div className="mb-6">
          {imageUrls.slice(0, 1).map((url, i) => (
            <img
              key={i}
              src={url}
              alt="Article header"
              loading="lazy"
              className="w-full rounded-lg object-cover max-h-80"
              onError={(e) => {
                (e.target as HTMLImageElement).style.display = "none";
              }}
            />
          ))}
        </div>
      )}

      {/* Rendered article content with typography */}
      <div
        ref={contentRef}
        className="prose prose-sm dark:prose-invert max-w-none
          prose-headings:text-foreground prose-headings:font-semibold
          prose-p:text-foreground/90 prose-p:leading-relaxed
          prose-a:text-primary prose-a:no-underline hover:prose-a:underline
          prose-img:rounded-lg prose-img:max-w-full
          prose-blockquote:border-l-primary/50 prose-blockquote:text-muted-foreground
          prose-code:text-primary prose-code:bg-muted prose-code:px-1 prose-code:rounded
          prose-pre:bg-muted prose-pre:text-foreground
          prose-strong:text-foreground prose-strong:font-semibold
          prose-li:text-foreground/90"
        dangerouslySetInnerHTML={{ __html: bodyHtml }}
      />
    </div>
  );
}
