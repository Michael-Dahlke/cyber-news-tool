import { useState, useEffect, useCallback } from "react";
import * as settingsService from "@/services/settingsService";
import type { Theme } from "@/types";

/**
 * Manages theme state — loads from settings on mount, listens for system
 * preference changes when set to "system", and applies the `dark` class
 * to `document.documentElement`.
 */
export function useTheme() {
  const [theme, setThemeState] = useState<Theme>("system");
  const [resolved, setResolved] = useState<"light" | "dark">("light");

  // Apply theme class to <html>
  const applyTheme = useCallback((t: Theme) => {
    const root = document.documentElement;
    let effective: "light" | "dark";

    if (t === "dark") {
      root.classList.add("dark");
      effective = "dark";
    } else if (t === "light") {
      root.classList.remove("dark");
      effective = "light";
    } else {
      const prefersDark = window.matchMedia(
        "(prefers-color-scheme: dark)"
      ).matches;
      if (prefersDark) {
        root.classList.add("dark");
        effective = "dark";
      } else {
        root.classList.remove("dark");
        effective = "light";
      }
    }

    setResolved(effective);
  }, []);

  // Load theme from backend on mount
  useEffect(() => {
    let cancelled = false;

    settingsService
      .getSettings()
      .then((settings) => {
        if (cancelled) return;
        setThemeState(settings.theme);
        applyTheme(settings.theme);
      })
      .catch(() => {
        // Fallback to system on error
        if (!cancelled) applyTheme("system");
      });

    return () => {
      cancelled = true;
    };
  }, [applyTheme]);

  // Listen for system color scheme changes when theme is "system"
  useEffect(() => {
    if (theme !== "system") return;

    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = (e: MediaQueryListEvent) => {
      const root = document.documentElement;
      if (e.matches) {
        root.classList.add("dark");
        setResolved("dark");
      } else {
        root.classList.remove("dark");
        setResolved("light");
      }
    };

    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, [theme]);

  // Public setter — applies immediately
  const setTheme = useCallback(
    (t: Theme) => {
      setThemeState(t);
      applyTheme(t);
    },
    [applyTheme]
  );

  return { theme, resolved, setTheme } as const;
}
