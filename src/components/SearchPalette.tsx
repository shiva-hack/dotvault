import { useState, useEffect, useRef, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { api, type SearchResult } from "../lib/tauri";

interface SearchPaletteProps {
  open: boolean;
  onClose: () => void;
}

export function SearchPalette({ open, onClose }: SearchPaletteProps) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [loading, setLoading] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const navigate = useNavigate();

  // Global keyboard shortcut
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        if (open) {
          onClose();
        } else {
          // parent handles opening
        }
      }
      if (e.key === "Escape" && open) {
        onClose();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open, onClose]);

  // Focus input when opened
  useEffect(() => {
    if (open) {
      setQuery("");
      setResults([]);
      setSelectedIndex(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [open]);

  // Debounced search
  const doSearch = useCallback(async (q: string) => {
    if (q.length < 2) {
      setResults([]);
      return;
    }
    setLoading(true);
    try {
      const res = await api.search(q);
      setResults(res);
      setSelectedIndex(0);
    } catch (e) {
      console.error("Search failed:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    const timer = setTimeout(() => doSearch(query), 200);
    return () => clearTimeout(timer);
  }, [query, doSearch]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((i) => Math.min(i + 1, results.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((i) => Math.max(i - 1, 0));
    } else if (e.key === "Enter" && results[selectedIndex]) {
      onClose();
      navigate(`/file/${results[selectedIndex].file_id}`);
    }
  };

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-[20vh]">
      {/* Backdrop */}
      <div className="absolute inset-0 bg-black/60" onClick={onClose} />

      {/* Palette */}
      <div className="relative w-full max-w-xl bg-surface border border-border rounded-xl shadow-2xl overflow-hidden">
        {/* Input */}
        <div className="flex items-center gap-3 px-4 border-b border-border">
          <span className="text-muted">🔍</span>
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search variables, projects, environments..."
            className="flex-1 py-3 bg-transparent text-white text-sm outline-none placeholder:text-muted"
          />
          {loading && (
            <div className="w-4 h-4 border-2 border-accent border-t-transparent rounded-full animate-spin" />
          )}
        </div>

        {/* Results */}
        <div className="max-h-80 overflow-y-auto">
          {results.length === 0 && query.length >= 2 && !loading && (
            <div className="px-4 py-8 text-center text-muted text-sm">
              No results for "{query}"
            </div>
          )}

          {results.map((result, idx) => (
            <button
              key={`${result.var_id}`}
              onClick={() => {
                onClose();
                navigate(`/file/${result.file_id}`);
              }}
              className={`w-full text-left px-4 py-2.5 flex items-center gap-3 text-sm ${
                idx === selectedIndex
                  ? "bg-accent/10 text-white"
                  : "text-zinc-400 hover:bg-surface-2"
              }`}
            >
              <span className="font-mono text-accent text-xs bg-surface-2 px-1.5 py-0.5 rounded">
                {result.key}
              </span>
              <span className="text-muted text-xs">in</span>
              <span className="text-xs truncate">{result.project_name}</span>
              <span className="text-xs text-muted">→</span>
              <TierBadge tier={result.tier} />
            </button>
          ))}
        </div>

        {/* Footer hint */}
        <div className="px-4 py-2 border-t border-border flex items-center gap-4 text-[10px] text-muted">
          <span><kbd className="bg-surface-2 px-1 rounded">↑↓</kbd> navigate</span>
          <span><kbd className="bg-surface-2 px-1 rounded">↵</kbd> open</span>
          <span><kbd className="bg-surface-2 px-1 rounded">esc</kbd> close</span>
        </div>
      </div>
    </div>
  );
}

function TierBadge({ tier }: { tier: string }) {
  const colors: Record<string, string> = {
    base: "bg-zinc-700 text-zinc-300",
    local: "bg-blue-500/20 text-blue-400",
    development: "bg-green/20 text-green",
    staging: "bg-yellow/20 text-yellow",
    production: "bg-red/20 text-red",
  };
  return (
    <span
      className={`text-[10px] px-1.5 py-0.5 rounded font-medium ${
        colors[tier] || "bg-surface-2 text-muted"
      }`}
    >
      {tier}
    </span>
  );
}
