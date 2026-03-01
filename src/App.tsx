import { BrowserRouter, Routes, Route } from "react-router-dom";
import { AppShell } from "@/components/Layout/AppShell";

function DashboardPlaceholder() {
  return (
    <div className="flex h-full items-center justify-center">
      <h1 className="text-2xl font-bold text-muted-foreground">Dashboard</h1>
    </div>
  );
}

function SearchPlaceholder() {
  return (
    <div className="flex h-full items-center justify-center">
      <h1 className="text-2xl font-bold text-muted-foreground">Search</h1>
    </div>
  );
}

function SettingsPlaceholder() {
  return (
    <div className="flex h-full items-center justify-center">
      <h1 className="text-2xl font-bold text-muted-foreground">Settings</h1>
    </div>
  );
}

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<AppShell />}>
          <Route path="/" element={<DashboardPlaceholder />} />
          <Route path="/search" element={<SearchPlaceholder />} />
          <Route path="/settings" element={<SettingsPlaceholder />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
