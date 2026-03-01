import { BrowserRouter, Routes, Route } from "react-router-dom";
import { AppShell } from "@/components/Layout/AppShell";
import { ErrorBoundary } from "@/components/Layout/ErrorBoundary";
import { DashboardView } from "@/components/Dashboard/DashboardView";
import { ReaderView } from "@/components/Reader/ReaderView";
import { SearchResults } from "@/components/Search/SearchResults";
import { SettingsView } from "@/components/Settings/SettingsView";
import { SourcesView } from "@/components/Sources/SourcesView";
import { SourceFeedView } from "@/components/Sources/SourceFeedView";

export default function App() {
  return (
    <ErrorBoundary>
      <BrowserRouter>
        <Routes>
          <Route element={<AppShell />}>
            <Route path="/" element={<DashboardView />} />
            <Route path="/article/:articleId" element={<ReaderView />} />
            <Route path="/sources" element={<SourcesView />} />
            <Route path="/sources/:sourceId" element={<SourceFeedView />} />
            <Route path="/search" element={<SearchResults />} />
            <Route path="/settings" element={<SettingsView />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </ErrorBoundary>
  );
}
