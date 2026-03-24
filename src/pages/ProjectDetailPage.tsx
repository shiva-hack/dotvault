import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { api, type EnvFile, type Project } from "../lib/tauri";
import { TierBadge } from "../components/TierBadge";

export function ProjectDetailPage() {
  const { projectId } = useParams<{ projectId: string }>();
  const navigate = useNavigate();
  const [project, setProject] = useState<Project | null>(null);
  const [envFiles, setEnvFiles] = useState<EnvFile[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!projectId) return;
    loadProject();
  }, [projectId]);

  const loadProject = async () => {
    if (!projectId) return;
    setLoading(true);
    try {
      const [projects, files] = await Promise.all([
        api.getProjects(),
        api.getEnvFiles(projectId),
      ]);
      const p = projects.find((p) => p.id === projectId) || null;
      setProject(p);
      setEnvFiles(files);
    } catch (e) {
      console.error("Failed to load project:", e);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return <LoadingState />;
  }

  if (!project) {
    return <p className="text-muted">Project not found</p>;
  }

  // Build environment tree visualization
  const baseFiles = envFiles.filter((f) => f.depth === 0);
  const tierGroups = envFiles.reduce((acc, f) => {
    if (f.depth === 0) return acc;
    const key = f.tier;
    if (!acc[key]) acc[key] = [];
    acc[key].push(f);
    return acc;
  }, {} as Record<string, EnvFile[]>);

  return (
    <div>
      {/* Project header */}
      <div className="mb-6">
        <button
          onClick={() => navigate("/")}
          className="text-sm text-muted hover:text-white mb-2 inline-block"
        >
          ← All Projects
        </button>
        <div className="flex items-center gap-3">
          <h1 className="text-xl font-semibold text-white">{project.name}</h1>
          {project.ecosystem && (
            <span className="text-xs bg-surface-2 text-muted px-2 py-0.5 rounded-full">
              {project.ecosystem}
            </span>
          )}
        </div>
        <p className="text-sm text-muted font-mono mt-1">{project.root_path}</p>
      </div>

      {/* Actions */}
      <div className="flex gap-2 mb-6">
        <button
          onClick={() => navigate(`/compare/${projectId}`)}
          disabled={envFiles.length < 2}
          className="px-4 py-2 bg-surface border border-border hover:border-accent/50 rounded-lg text-sm text-zinc-300 disabled:opacity-40 disabled:cursor-not-allowed"
        >
          Compare Environments
        </button>
        <button
          onClick={loadProject}
          className="px-4 py-2 bg-surface border border-border hover:border-accent/50 rounded-lg text-sm text-zinc-300"
        >
          Re-scan
        </button>
      </div>

      {/* Environment hierarchy tree */}
      <div className="bg-surface border border-border rounded-lg p-4 mb-6">
        <h2 className="text-sm font-medium text-white mb-3">Environment Hierarchy</h2>
        <div className="font-mono text-sm space-y-1">
          {baseFiles.map((f) => (
            <TreeNode key={f.id} file={f} onClick={() => navigate(`/file/${f.id}`)} depth={0} />
          ))}
          {Object.entries(tierGroups).map(([tier, files]) => (
            <div key={tier}>
              {files
                .sort((a, b) => a.depth - b.depth)
                .map((f) => (
                  <TreeNode
                    key={f.id}
                    file={f}
                    onClick={() => navigate(`/file/${f.id}`)}
                    depth={f.depth}
                  />
                ))}
            </div>
          ))}
        </div>
      </div>

      {/* Env files table */}
      <div>
        <h2 className="text-sm font-medium text-white mb-3">
          All Environment Files ({envFiles.length})
        </h2>
        <div className="bg-surface border border-border rounded-lg overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-border text-left">
                <th className="px-4 py-2.5 text-muted font-medium">File</th>
                <th className="px-4 py-2.5 text-muted font-medium">Tier</th>
                <th className="px-4 py-2.5 text-muted font-medium text-right">Variables</th>
                <th className="px-4 py-2.5 text-muted font-medium text-right">Size</th>
              </tr>
            </thead>
            <tbody>
              {envFiles.map((file) => (
                <tr
                  key={file.id}
                  onClick={() => navigate(`/file/${file.id}`)}
                  className="border-b border-border/50 hover:bg-surface-2 cursor-pointer"
                >
                  <td className="px-4 py-2.5">
                    <span className="font-mono text-white">{file.filename}</span>
                    {file.relative_path !== file.filename && (
                      <span className="text-muted ml-2 text-xs">{file.relative_path}</span>
                    )}
                  </td>
                  <td className="px-4 py-2.5">
                    <TierBadge tier={file.tier} depth={file.depth} />
                  </td>
                  <td className="px-4 py-2.5 text-right text-muted">{file.var_count}</td>
                  <td className="px-4 py-2.5 text-right text-muted">
                    {formatFileSize(file.file_size)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}

function TreeNode({
  file,
  onClick,
  depth,
}: {
  file: EnvFile;
  onClick: () => void;
  depth: number;
}) {
  const indent = depth > 0 ? "│   ".repeat(depth - 1) + (depth > 0 ? "├── " : "") : "";
  return (
    <button
      onClick={onClick}
      className="block w-full text-left hover:bg-surface-2 px-2 py-0.5 rounded text-zinc-400 hover:text-white"
    >
      <span className="text-muted">{indent}</span>
      <span className="text-accent">{file.filename}</span>
      <span className="text-muted ml-3 text-xs">
        {file.var_count} vars · {file.tier}
      </span>
    </button>
  );
}

function LoadingState() {
  return (
    <div className="flex items-center justify-center py-20">
      <div className="w-6 h-6 border-2 border-accent border-t-transparent rounded-full animate-spin" />
    </div>
  );
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
