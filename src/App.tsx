import { useState, useEffect } from "react";
import { Routes, Route, Navigate } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import { useVault } from "./hooks/useVault";
import { api } from "./lib/tauri";
import { Sidebar } from "./components/Sidebar";
import { Header } from "./components/Header";
import { SearchPalette } from "./components/SearchPalette";
import { SetupScreen } from "./pages/SetupScreen";
import { UnlockScreen } from "./pages/UnlockScreen";
import { ProjectsPage } from "./pages/ProjectsPage";
import { ProjectDetailPage } from "./pages/ProjectDetailPage";
import { EnvFilePage } from "./pages/EnvFilePage";
import { ComparisonPage } from "./pages/ComparisonPage";
import { SettingsPage } from "./pages/SettingsPage";

export default function App() {
  const vault = useVault();
  const [searchOpen, setSearchOpen] = useState(false);

  // Global Cmd/Ctrl+K shortcut
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        setSearchOpen((prev) => !prev);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  // Listen for file watcher events and auto-refresh
  useEffect(() => {
    if (!vault.isUnlocked) return;

    // Start the file watcher
    api.startWatcher().catch(console.error);

    const unlisten = listen<string>("env-file-changed", () => {
      vault.refresh();
    });

    return () => {
      api.stopWatcher().catch(console.error);
      unlisten.then((fn) => fn());
    };
  }, [vault.isUnlocked]);

  // Loading state
  if (vault.loading || vault.isSetup === null) {
    return (
      <div className="h-screen bg-bg flex items-center justify-center">
        <div className="flex flex-col items-center gap-4">
          <div className="w-8 h-8 border-2 border-accent border-t-transparent rounded-full animate-spin" />
          <p className="text-muted text-sm">Loading vault...</p>
        </div>
      </div>
    );
  }

  // First time setup
  if (!vault.isSetup) {
    return <SetupScreen onSetup={vault.setupVault} />;
  }

  // Locked — need to unlock
  if (!vault.isUnlocked) {
    return <UnlockScreen onUnlock={vault.unlock} />;
  }

  // Main app
  return (
    <div className="h-screen bg-bg flex overflow-hidden">
      <Sidebar
        roots={vault.roots}
        projects={vault.projects}
        onRefresh={vault.refresh}
        onLock={vault.lock}
      />

      <div className="flex-1 flex flex-col min-w-0">
        <Header onSearchOpen={() => setSearchOpen(true)} onScanAll={vault.refresh} />

        <main className="flex-1 overflow-y-auto p-6">
          <Routes>
            <Route
              path="/"
              element={
                <ProjectsPage
                  projects={vault.projects}
                  roots={vault.roots}
                  onRefresh={vault.refresh}
                />
              }
            />
            <Route
              path="/project/:projectId"
              element={<ProjectDetailPage />}
            />
            <Route
              path="/file/:fileId"
              element={<EnvFilePage />}
            />
            <Route
              path="/compare/:projectId"
              element={<ComparisonPage />}
            />
            <Route
              path="/settings"
              element={
                <SettingsPage
                  onLock={vault.lock}
                  roots={vault.roots}
                  onRefresh={vault.refresh}
                />
              }
            />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </main>
      </div>

      <SearchPalette open={searchOpen} onClose={() => setSearchOpen(false)} />
    </div>
  );
}
