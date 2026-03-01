import { createContext, useContext } from "react";
import type { Theme } from "@/types";

export interface ThemeContextValue {
  theme: Theme;
  resolved: "light" | "dark";
  setTheme: (t: Theme) => void;
}

export const ThemeContext = createContext<ThemeContextValue>({
  theme: "system",
  resolved: "light",
  setTheme: () => {},
});

export function useThemeContext() {
  return useContext(ThemeContext);
}
