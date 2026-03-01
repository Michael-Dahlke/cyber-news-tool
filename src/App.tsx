import { BrowserRouter, Routes, Route } from "react-router-dom";
import { AppShell } from "@/components/Layout/AppShell";
import { DashboardView } from "@/components/Dashboard/DashboardView";
import { ReaderView } from "@/components/Reader/ReaderView";
import { SettingsView } from "@/components/Settings/SettingsView";

function SearchPlaceholder() {
  return (
    <div className="flex h-full items-center justify-center">
      <h1 className="text-2xl font-bold text-muted-foreground">Search</h1>
    </div>
  );
}

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<AppShell />}>
          <Route path="/" element={<DashboardView />} />
          <Route path="/article/:articleId" element={<ReaderView />} />
          <Route path="/search" element={<SearchPlaceholder />} />
          <Route path="/settings" element={<SettingsView />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
