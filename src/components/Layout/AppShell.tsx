import { Outlet } from "react-router-dom";
import { Sidebar } from "./Sidebar";
import { useTheme } from "@/hooks/useTheme";
import { ThemeContext } from "@/hooks/ThemeContext";

export function AppShell() {
  const themeValue = useTheme();

  return (
    <ThemeContext.Provider value={themeValue}>
      <div className="flex h-screen w-screen overflow-hidden bg-background text-foreground">
        <Sidebar />
        <main id="app-scroll-container" className="flex-1 overflow-y-auto">
          <Outlet />
        </main>
      </div>
    </ThemeContext.Provider>
  );
}
