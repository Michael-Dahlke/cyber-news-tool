import { useCallback, useEffect, useRef, useState } from "react";
import { X, ZoomIn, ZoomOut } from "lucide-react";

interface ImageLightboxProps {
  src: string;
  alt?: string;
  onClose: () => void;
}

const MIN_SCALE = 1;
const MAX_SCALE = 4;
const ZOOM_STEP = 0.5;

/**
 * Full-screen lightbox for article images (FR-044 / FR-045).
 * Supports pinch-to-zoom via trackpad (wheel + ctrlKey) and
 * scroll-to-pan when zoomed beyond 100%.
 */
export function ImageLightbox({ src, alt, onClose }: ImageLightboxProps) {
  const [scale, setScale] = useState(1);
  const contentRef = useRef<HTMLDivElement>(null);

  // Close on Escape
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  // Handle trackpad pinch-to-zoom (ctrl + wheel)
  const handleWheel = useCallback(
    (e: React.WheelEvent) => {
      if (e.ctrlKey || e.metaKey) {
        e.preventDefault();
        setScale((prev) => {
          const next = prev - e.deltaY * 0.01;
          return Math.min(MAX_SCALE, Math.max(MIN_SCALE, next));
        });
      }
    },
    []
  );

  const zoomIn = () => setScale((s) => Math.min(MAX_SCALE, s + ZOOM_STEP));
  const zoomOut = () => setScale((s) => Math.max(MIN_SCALE, s - ZOOM_STEP));
  const resetZoom = () => setScale(1);

  return (
    <div
      className="fixed inset-0 z-50 flex flex-col bg-black/90"
      role="dialog"
      aria-modal="true"
      aria-label="Image lightbox"
    >
      {/* Toolbar */}
      <div className="flex items-center justify-between px-4 py-2">
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={zoomOut}
            disabled={scale <= MIN_SCALE}
            className="inline-flex h-8 w-8 items-center justify-center rounded-md text-white/80 hover:bg-white/10 disabled:opacity-30"
            title="Zoom out"
            aria-label="Zoom out"
          >
            <ZoomOut className="h-4 w-4" />
          </button>
          <button
            type="button"
            onClick={resetZoom}
            className="inline-flex h-8 min-w-[3.5rem] items-center justify-center rounded-md px-2 text-xs font-medium text-white/80 hover:bg-white/10"
            title="Reset zoom"
          >
            {Math.round(scale * 100)}%
          </button>
          <button
            type="button"
            onClick={zoomIn}
            disabled={scale >= MAX_SCALE}
            className="inline-flex h-8 w-8 items-center justify-center rounded-md text-white/80 hover:bg-white/10 disabled:opacity-30"
            title="Zoom in"
            aria-label="Zoom in"
          >
            <ZoomIn className="h-4 w-4" />
          </button>
        </div>

        <button
          type="button"
          onClick={onClose}
          className="inline-flex h-8 w-8 items-center justify-center rounded-md text-white/80 hover:bg-white/10"
          title="Close"
          aria-label="Close lightbox"
        >
          <X className="h-5 w-5" />
        </button>
      </div>

      {/* Scrollable image container */}
      <div
        ref={contentRef}
        className="flex-1 overflow-auto"
        onWheel={handleWheel}
      >
        <div className="flex min-h-full items-center justify-center p-4">
          <img
            src={src}
            alt={alt ?? "Full-size image"}
            className="max-w-none select-none"
            style={{
              transform: `scale(${scale})`,
              transformOrigin: "center center",
            }}
            draggable={false}
          />
        </div>
      </div>
    </div>
  );
}
