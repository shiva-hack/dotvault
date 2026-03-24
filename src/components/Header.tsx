import { api } from "../lib/tauri";
import { useState } from "react";

interface HeaderProps {
  onSearchOpen: () => void;
  onScanAll: () => void;
}

export function Header({ onSearchOpen, onScanAll }: HeaderProps) {
  const [scanning, setScanning] = useState(false);

  const handleScan = async () => {
    setScanning(true);
    try {
      await api.scanAll();
      onScanAll();
    } catch (e) {
      console.error("Scan failed:", e);
    } finally {
      setScanning(false);
    }
  };

  return (
    <header className="h-12 border-b border-border flex items-center justify-between px-4 shrink-0 bg-surface">
      <div className="flex-1" />

      <div className="flex items-center gap-2">
        {/* Search trigger */}
        <button
          onClick={onSearchOpen}
          className="flex items-center gap-2 px-3 py-1.5 bg-surface-2 hover:bg-border rounded-md text-sm text-muted"
        >
          <span>Search</span>
          <kbd className="text-[10px] bg-bg px-1.5 py-0.5 rounded font-mono">⌘K</kbd>
        </button>

        {/* Scan button */}
        <button
          onClick={handleScan}
          disabled={scanning}
          className="flex items-center gap-1.5 px-3 py-1.5 bg-surface-2 hover:bg-border rounded-md text-sm text-muted disabled:opacity-50"
        >
          <span className={scanning ? "animate-spin" : ""}>⟳</span>
          <span>{scanning ? "Scanning..." : "Scan"}</span>
        </button>
      </div>
    </header>
  );
}
