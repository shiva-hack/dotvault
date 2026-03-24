import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { api, type EnvFile, type ComparisonMatrix } from "../lib/tauri";
import { TierBadge } from "../components/TierBadge";

export function ComparisonPage() {
  const { projectId } = useParams<{ projectId: string }>();
  const navigate = useNavigate();
  const [envFiles, setEnvFiles] = useState<EnvFile[]>([]);
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
  const [matrix, setMatrix] = useState<ComparisonMatrix | null>(null);
  const [loading, setLoading] = useState(true);
  const [comparing, setComparing] = useState(false);

  useEffect(() => {
    if (!projectId) return;
    loadFiles();
  }, [projectId]);

  const loadFiles = async () => {
    if (!projectId) return;
    setLoading(true);
    try {
      const files = await api.getEnvFiles(projectId);
      setEnvFiles(files);
      // Auto-select all files for comparison
      const ids = files.map((f) => f.id);
      setSelectedFiles(ids);
      if (ids.length >= 2) {
        await runComparison(ids);
      }
    } catch (e) {
      console.error("Failed to load:", e);
    } finally {
      setLoading(false);
    }
  };

  const runComparison = async (fileIds: string[]) => {
    if (fileIds.length < 2) return;
    setComparing(true);
    try {
      const result = await api.compareEnvs(fileIds);
      setMatrix(result);
    } catch (e) {
      console.error("Comparison failed:", e);
    } finally {
      setComparing(false);
    }
  };

  const toggleFile = (fileId: string) => {
    setSelectedFiles((prev) => {
      const next = prev.includes(fileId)
        ? prev.filter((id) => id !== fileId)
        : [...prev, fileId];
      if (next.length >= 2) {
        runComparison(next);
      } else {
        setMatrix(null);
      }
      return next;
    });
  };

  const copyAsMarkdown = () => {
    if (!matrix) return;
    let md = "| Key | " + matrix.files.map((f) => f.filename).join(" | ") + " |\n";
    md += "| --- | " + matrix.files.map(() => "---").join(" | ") + " |\n";
    for (const key of matrix.keys) {
      md +=
        "| " +
        key.key +
        " | " +
        key.presence.map((p) => (p ? "✓" : "✗")).join(" | ") +
        " |\n";
    }
    navigator.clipboard.writeText(md);
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-20">
        <div className="w-6 h-6 border-2 border-accent border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  return (
    <div>
      <div className="mb-6">
        <button
          onClick={() => navigate(-1)}
          className="text-sm text-muted hover:text-white mb-2 inline-block"
        >
          ← Back to Project
        </button>
        <h1 className="text-xl font-semibold text-white">Cross-Environment Comparison</h1>
        <p className="text-sm text-muted mt-1">
          See which variables exist across environments
        </p>
      </div>

      {/* File selector */}
      <div className="flex flex-wrap gap-2 mb-6">
        {envFiles.map((file) => (
          <button
            key={file.id}
            onClick={() => toggleFile(file.id)}
            className={`px-3 py-1.5 rounded-lg text-sm border ${
              selectedFiles.includes(file.id)
                ? "border-accent bg-accent/10 text-white"
                : "border-border bg-surface text-muted"
            }`}
          >
            <span className="font-mono">{file.filename}</span>
            <TierBadge tier={file.tier} />
          </button>
        ))}
      </div>

      {/* Export button */}
      {matrix && (
        <div className="flex justify-end mb-3">
          <button
            onClick={copyAsMarkdown}
            className="text-xs text-muted hover:text-white px-3 py-1.5 bg-surface border border-border rounded hover:border-accent/50"
          >
            Copy as Markdown
          </button>
        </div>
      )}

      {/* Comparison matrix */}
      {comparing ? (
        <div className="flex items-center justify-center py-12">
          <div className="w-6 h-6 border-2 border-accent border-t-transparent rounded-full animate-spin" />
        </div>
      ) : matrix ? (
        <div className="bg-surface border border-border rounded-lg overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-border">
                <th className="px-4 py-3 text-left text-muted font-medium sticky left-0 bg-surface">
                  Variable
                </th>
                {matrix.files.map((file) => (
                  <th key={file.file_id} className="px-4 py-3 text-center">
                    <div className="text-white font-mono text-xs">{file.filename}</div>
                    <TierBadge tier={file.tier} />
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {matrix.keys.map((key) => (
                <tr
                  key={key.key}
                  className={`border-b border-border/50 ${
                    key.status === "single"
                      ? "bg-red/5"
                      : key.status === "some"
                      ? "bg-yellow/5"
                      : ""
                  }`}
                >
                  <td className="px-4 py-2 font-mono text-white sticky left-0 bg-surface">
                    <div className="flex items-center gap-2">
                      <StatusDot status={key.status} />
                      {key.key}
                    </div>
                  </td>
                  {key.presence.map((present, idx) => (
                    <td key={idx} className="px-4 py-2 text-center">
                      {present ? (
                        <span className="text-green text-lg">✓</span>
                      ) : (
                        <span className="text-red/50 text-lg">✗</span>
                      )}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>

          {matrix.keys.length === 0 && (
            <div className="px-4 py-8 text-center text-muted text-sm">
              No variables found across selected files
            </div>
          )}
        </div>
      ) : (
        <div className="text-center py-12 text-muted text-sm">
          Select at least 2 env files to compare
        </div>
      )}

      {/* Legend */}
      {matrix && matrix.keys.length > 0 && (
        <div className="flex items-center gap-6 mt-4 text-xs text-muted">
          <span className="flex items-center gap-1.5">
            <StatusDot status="all" /> Present in all environments
          </span>
          <span className="flex items-center gap-1.5">
            <StatusDot status="some" /> Missing in some environments
          </span>
          <span className="flex items-center gap-1.5">
            <StatusDot status="single" /> Only in one environment
          </span>
        </div>
      )}
    </div>
  );
}

function StatusDot({ status }: { status: string }) {
  const colors = {
    all: "bg-green",
    some: "bg-yellow",
    single: "bg-red",
  };
  return (
    <span
      className={`inline-block w-2 h-2 rounded-full ${
        colors[status as keyof typeof colors] || "bg-muted"
      }`}
    />
  );
}
