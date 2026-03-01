import { useRef, useEffect, useState, useCallback } from "react";
import { ImageLightbox } from "./ImageLightbox";

interface ContentRendererProps {
  bodyHtml: string;
  imageUrls: string[];
  /** Reader font size in px (FR-043) */
  fontSize?: number;
}

/**
 * Renders cleaned article HTML with inline images and typography styles.
 * Uses dangerouslySetInnerHTML for the extracted content, with a prose
 * container for consistent typography.
 *
 * Applies `--reader-font-size` CSS variable and opens ImageLightbox
 * when an image is clicked (FR-044).
 */
export function ContentRenderer({ bodyHtml, imageUrls, fontSize = 16 }: ContentRendererProps) {
  const contentRef = useRef<HTMLDivElement>(null);
  const [lightboxSrc, setLightboxSrc] = useState<string | null>(null);

  const handleImageClick = useCallback((e: MouseEvent) => {
    const target = e.target as HTMLElement;
    if (target.tagName === "IMG") {
      const src = (target as HTMLImageElement).src;
      if (src) setLightboxSrc(src);
    }
  }, []);

  useEffect(() => {
    if (!contentRef.current) return;

    // Ensure all images have proper loading and alt attributes
    const images = contentRef.current.querySelectorAll("img");
    images.forEach((img) => {
      img.setAttribute("loading", "lazy");
      if (!img.alt) {
        img.alt = "Article image";
      }
      // Add responsive styling + clickable cursor
      img.classList.add("max-w-full", "h-auto", "rounded-lg", "my-4", "cursor-pointer");
    });

    // Make links open externally
    const links = contentRef.current.querySelectorAll("a");
    links.forEach((link) => {
      link.setAttribute("target", "_blank");
      link.setAttribute("rel", "noopener noreferrer");
      link.classList.add("text-primary", "underline", "hover:text-primary/80");
    });

    // Attach click handler for lightbox
    const el = contentRef.current;
    el.addEventListener("click", handleImageClick);
    return () => el.removeEventListener("click", handleImageClick);
  }, [bodyHtml, handleImageClick]);

  return (
    <div className="mx-auto max-w-3xl" style={{ ["--reader-font-size" as string]: `${fontSize}px` }}>
      {/* Image gallery if article has notable images */}
      {imageUrls.length > 0 && (
        <div className="mb-6">
          {imageUrls.slice(0, 1).map((url, i) => (
            <img
              key={i}
              src={url}
              alt="Article header"
              loading="lazy"
              className="w-full rounded-lg object-cover max-h-80 cursor-pointer"
              onClick={() => setLightboxSrc(url)}
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
        style={{ fontSize: `${fontSize}px` }}
        dangerouslySetInnerHTML={{ __html: bodyHtml }}
      />

      {/* Lightbox overlay */}
      {lightboxSrc && (
        <ImageLightbox
          src={lightboxSrc}
          onClose={() => setLightboxSrc(null)}
        />
      )}
    </div>
  );
}
