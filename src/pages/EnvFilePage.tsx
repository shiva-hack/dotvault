import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { api, type EnvVar, type EnvFile } from "../lib/tauri";
import { TierBadge } from "../components/TierBadge";

export function EnvFilePage() {
  const { fileId } = useParams<{ fileId: string }>();
  const navigate = useNavigate();
  const [file, setFile] = useState<EnvFile | null>(null);
  const [variables, setVariables] = useState<EnvVar[]>([]);
  const [revealedIds, setRevealedIds] = useState<Set<string>>(new Set());
  const [decryptedValues, setDecryptedValues] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!fileId) return;
    loadFile();
  }, [fileId]);

  const loadFile = async () => {
    if (!fileId) return;
    setLoading(true);
    try {
      const vars = await api.getEnvVariables(fileId);
      setVariables(vars);

      // Find the file metadata
      const projects = await api.getProjects();
      for (const project of projects) {
        const files = await api.getEnvFiles(project.id);
        const f = files.find((f) => f.id === fileId);
        if (f) {
          setFile(f);
          break;
        }
      }
    } catch (e) {
      console.error("Failed to load file:", e);
    } finally {
      setLoading(false);
    }
  };

  const toggleReveal = async (variable: EnvVar) => {
    const newRevealed = new Set(revealedIds);

    if (revealedIds.has(variable.id)) {
      newRevealed.delete(variable.id);
      setRevealedIds(newRevealed);
      return;
    }

    // Decrypt the value
    if (!decryptedValues[variable.id]) {
      try {
        const value = await api.decryptValue(
          variable.encrypted_value,
          variable.nonce
        );
        setDecryptedValues((prev) => ({ ...prev, [variable.id]: value }));
      } catch (e) {
        console.error("Failed to decrypt:", e);
        return;
      }
    }

    newRevealed.add(variable.id);
    setRevealedIds(newRevealed);
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
      {/* Header */}
      <div className="mb-6">
        <button
          onClick={() => navigate(-1)}
          className="text-sm text-muted hover:text-white mb-2 inline-block"
        >
          ← Back
        </button>

        {file && (
          <div className="flex items-center gap-3">
            <h1 className="text-xl font-semibold text-white font-mono">{file.filename}</h1>
            <TierBadge tier={file.tier} depth={file.depth} />
          </div>
        )}

        {file && (
          <div className="flex items-center gap-4 mt-2 text-sm text-muted">
            <span>{file.var_count} variables</span>
            <span>{formatFileSize(file.file_size)}</span>
            <span className="font-mono text-xs">{file.relative_path}</span>
          </div>
        )}
      </div>

      {/* Variables list */}
      <div className="bg-surface border border-border rounded-lg overflow-hidden">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-border text-left">
              <th className="px-4 py-2.5 text-muted font-medium w-8">#</th>
              <th className="px-4 py-2.5 text-muted font-medium">Key</th>
              <th className="px-4 py-2.5 text-muted font-medium">Value</th>
              <th className="px-4 py-2.5 text-muted font-medium w-20">Action</th>
            </tr>
          </thead>
          <tbody>
            {variables.map((variable) => (
              <tr key={variable.id} className="border-b border-border/50 group">
                <td className="px-4 py-2.5 text-muted/50 text-xs font-mono">
                  {variable.line_number}
                </td>
                <td className="px-4 py-2.5">
                  {variable.comment && (
                    <p className="text-muted/60 text-xs mb-0.5 font-mono">{variable.comment}</p>
                  )}
                  <span className="font-mono text-accent font-medium">{variable.key}</span>
                </td>
                <td className="px-4 py-2.5 font-mono">
                  {revealedIds.has(variable.id) ? (
                    <span className="text-white text-xs break-all">
                      {decryptedValues[variable.id] || ""}
                    </span>
                  ) : (
                    <span className="text-muted">••••••••••••</span>
                  )}
                </td>
                <td className="px-4 py-2.5">
                  <button
                    onClick={() => toggleReveal(variable)}
                    className="text-xs text-muted hover:text-white px-2 py-1 rounded hover:bg-surface-2"
                  >
                    {revealedIds.has(variable.id) ? "Hide" : "Reveal"}
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>

        {variables.length === 0 && (
          <div className="px-4 py-8 text-center text-muted text-sm">
            No variables found in this file
          </div>
        )}
      </div>
    </div>
  );
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
