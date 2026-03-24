import { useNavigate, useLocation } from "react-router-dom";
import { api, type Root, type Project } from "../lib/tauri";
import { open } from "@tauri-apps/plugin-dialog";

interface SidebarProps {
  roots: Root[];
  projects: Project[];
  onRefresh: () => void;
  onLock: () => void;
}

const ecosystemIcons: Record<string, string> = {
  node: "JS",
  rust: "Rs",
  python: "Py",
  go: "Go",
  ruby: "Rb",
  php: "PH",
  dotnet: ".N",
  unknown: "??",
};

export function Sidebar({ roots, projects, onRefresh, onLock }: SidebarProps) {
  const navigate = useNavigate();
  const location = useLocation();

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

  const handleRemoveRoot = async (path: string) => {
    try {
      await api.removeRoot(path);
      onRefresh();
    } catch (e) {
      console.error("Failed to remove root:", e);
    }
  };

  const projectsByRoot = roots.map((root) => ({
    root,
    projects: projects.filter((p) => p.root_path === root.path),
  }));

  return (
    <aside className="w-64 bg-surface border-r border-border flex flex-col h-full shrink-0">
      {/* Logo */}
      <div className="p-4 border-b border-border flex items-center gap-2">
        <span className="text-lg">🔐</span>
        <h1 className="font-semibold text-sm tracking-tight">DotEnv Vault</h1>
      </div>

      {/* Navigation */}
      <nav className="flex-1 overflow-y-auto py-2">
        {/* Overview */}
        <button
          onClick={() => navigate("/")}
          className={`w-full text-left px-4 py-2 text-sm hover:bg-surface-2 ${
            location.pathname === "/" ? "bg-surface-2 text-white" : "text-muted"
          }`}
        >
          All Projects
        </button>

        {/* Roots and their projects */}
        {projectsByRoot.map(({ root, projects: rootProjects }) => (
          <div key={root.path} className="mt-3">
            <div className="px-4 flex items-center justify-between group">
              <span className="text-xs font-medium text-muted uppercase tracking-wider truncate flex-1">
                {root.path.split("/").pop() || root.path}
              </span>
              <button
                onClick={() => handleRemoveRoot(root.path)}
                className="text-muted hover:text-red opacity-0 group-hover:opacity-100 text-xs ml-1"
                title="Remove root"
              >
                ×
              </button>
            </div>

            {rootProjects.length === 0 ? (
              <p className="px-4 py-1 text-xs text-muted/50 italic">No projects found</p>
            ) : (
              rootProjects.map((project) => (
                <button
                  key={project.id}
                  onClick={() => navigate(`/project/${project.id}`)}
                  className={`w-full text-left px-4 py-1.5 text-sm hover:bg-surface-2 flex items-center gap-2 ${
                    location.pathname === `/project/${project.id}`
                      ? "bg-surface-2 text-white"
                      : "text-zinc-400"
                  }`}
                >
                  <span className="text-[10px] font-mono bg-surface-2 px-1 rounded text-accent">
                    {ecosystemIcons[project.ecosystem || "unknown"] || "??"}
                  </span>
                  <span className="truncate">{project.name}</span>
                  <span className="text-xs text-muted ml-auto">{project.env_file_count}</span>
                </button>
              ))
            )}
          </div>
        ))}

        {roots.length === 0 && (
          <div className="px-4 py-8 text-center">
            <p className="text-muted text-sm mb-3">No root directories added</p>
            <p className="text-muted/60 text-xs">Click "Add Root" below to start scanning</p>
          </div>
        )}
      </nav>

      {/* Bottom actions */}
      <div className="border-t border-border p-3 space-y-2">
        <button
          onClick={handleAddRoot}
          className="w-full py-2 bg-accent hover:bg-accent-hover text-white text-sm font-medium rounded-md"
        >
          + Add Root
        </button>
        <div className="flex gap-2">
          <button
            onClick={() => navigate("/settings")}
            className="flex-1 py-1.5 text-xs text-muted hover:text-white hover:bg-surface-2 rounded"
          >
            Settings
          </button>
          <button
            onClick={onLock}
            className="flex-1 py-1.5 text-xs text-muted hover:text-white hover:bg-surface-2 rounded"
          >
            Lock
          </button>
        </div>
      </div>
    </aside>
  );
}
