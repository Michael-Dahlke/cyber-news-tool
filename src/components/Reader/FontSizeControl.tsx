import { useState, useEffect } from "react";
import { Minus, Plus, Type } from "lucide-react";
import * as settingsService from "@/services/settingsService";

interface FontSizeControlProps {
  /** Current font size in px (from AppSettings) */
  initialSize: number;
  /** Called when font size changes — parent applies the CSS variable */
  onChange: (size: number) => void;
}

const MIN_FONT = 12;
const MAX_FONT = 28;
const STEP = 2;

/**
 * Increment/decrement stepper for reader font size (FR-043).
 * Persists each change to backend settings.
 */
export function FontSizeControl({ initialSize, onChange }: FontSizeControlProps) {
  const [size, setSize] = useState(initialSize);

  useEffect(() => {
    setSize(initialSize);
  }, [initialSize]);

  const update = (next: number) => {
    setSize(next);
    onChange(next);
    // Fire-and-forget persistence; errors are non-critical
    settingsService.updateSettings({ reader_font_size: next }).catch(() => {});
  };

  const decrement = () => {
    if (size - STEP >= MIN_FONT) update(size - STEP);
  };

  const increment = () => {
    if (size + STEP <= MAX_FONT) update(size + STEP);
  };

  return (
    <div className="inline-flex items-center gap-1 rounded-md border border-border">
      <button
        type="button"
        onClick={decrement}
        disabled={size <= MIN_FONT}
        className="inline-flex h-7 w-7 items-center justify-center text-muted-foreground hover:bg-accent hover:text-foreground disabled:opacity-40"
        title="Decrease font size"
        aria-label="Decrease font size"
      >
        <Minus className="h-3.5 w-3.5" />
      </button>
      <span
        className="inline-flex h-7 min-w-[3rem] items-center justify-center text-xs font-medium text-muted-foreground select-none"
        title="Current font size"
      >
        <Type className="mr-0.5 h-3 w-3" />
        {size}px
      </span>
      <button
        type="button"
        onClick={increment}
        disabled={size >= MAX_FONT}
        className="inline-flex h-7 w-7 items-center justify-center text-muted-foreground hover:bg-accent hover:text-foreground disabled:opacity-40"
        title="Increase font size"
        aria-label="Increase font size"
      >
        <Plus className="h-3.5 w-3.5" />
      </button>
    </div>
  );
}
