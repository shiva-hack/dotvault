import { useNavigate } from "react-router-dom";
import { api, type Project, type Root } from "../lib/tauri";
import { open } from "@tauri-apps/plugin-dialog";

interface ProjectsPageProps {
  projects: Project[];
  roots: Root[];
  onRefresh: () => void;
}

const ecosystemLabels: Record<string, string> = {
  node: "Node.js",
  rust: "Rust",
  python: "Python",
  go: "Go",
  ruby: "Ruby",
  php: "PHP",
  dotnet: ".NET",
  unknown: "Other",
};

export function ProjectsPage({ projects, roots, onRefresh }: ProjectsPageProps) {
  const navigate = useNavigate();

  const handleAddRoot = async () => {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      try {
        await api.addRoot(selected as string);
        onRefresh();
      } catch (e) {
        console.error("Failed to add root:", e);
      }
    }
  };

  const totalEnvFiles = projects.reduce((sum, p) => sum + p.env_file_count, 0);

  return (
    <div>
      {/* Stats row */}
      <div className="grid grid-cols-3 gap-4 mb-8">
        <StatCard label="Root Directories" value={roots.length} />
        <StatCard label="Projects" value={projects.length} />
        <StatCard label="Env Files" value={totalEnvFiles} />
      </div>

      {/* Projects grid */}
      {projects.length === 0 ? (
        <EmptyState onAdd={handleAddRoot} />
      ) : (
        <div>
          <h2 className="text-lg font-semibold text-white mb-4">All Projects</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            {projects.map((project) => (
              <button
                key={project.id}
                onClick={() => navigate(`/project/${project.id}`)}
                className="text-left bg-surface border border-border rounded-lg p-4 hover:border-accent/50 group"
              >
                <div className="flex items-start justify-between mb-2">
                  <h3 className="font-medium text-white group-hover:text-accent text-sm truncate">
                    {project.name}
                  </h3>
                  <span className="text-xs bg-surface-2 text-muted px-2 py-0.5 rounded-full ml-2 shrink-0">
                    {ecosystemLabels[project.ecosystem || "unknown"] || project.ecosystem}
                  </span>
                </div>

                <p className="text-xs text-muted truncate mb-3 font-mono">
                  {project.root_path}
                </p>

                <div className="flex items-center gap-3 text-xs text-muted">
                  <span>{project.env_file_count} env file{project.env_file_count !== 1 ? "s" : ""}</span>
                  {project.last_scanned && (
                    <span>
                      Scanned {formatRelativeTime(project.last_scanned)}
                    </span>
                  )}
                </div>
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

function StatCard({ label, value }: { label: string; value: number }) {
  return (
    <div className="bg-surface border border-border rounded-lg p-4">
      <p className="text-2xl font-semibold text-white">{value}</p>
      <p className="text-sm text-muted">{label}</p>
    </div>
  );
}

function EmptyState({ onAdd }: { onAdd: () => void }) {
  return (
    <div className="text-center py-16">
      <div className="text-4xl mb-4">📁</div>
      <h2 className="text-lg font-medium text-white mb-2">No projects yet</h2>
      <p className="text-muted text-sm mb-6 max-w-md mx-auto">
        Add a root directory to start scanning for projects and their .env files.
        dotvault will recursively discover all projects and their environment configurations.
      </p>
      <button
        onClick={onAdd}
        className="px-6 py-2.5 bg-accent hover:bg-accent-hover text-white font-medium rounded-lg text-sm"
      >
        + Add Root Directory
      </button>
    </div>
  );
}

function formatRelativeTime(timestamp: number): string {
  const now = Math.floor(Date.now() / 1000);
  const diff = now - timestamp;
  if (diff < 60) return "just now";
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  return `${Math.floor(diff / 86400)}d ago`;
}
