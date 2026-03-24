import { invoke } from "@tauri-apps/api/core";

// ── Types ────────────────────────────────────────────────────────────

export interface Root {
  path: string;
  added_at: number;
}

export interface Project {
  id: string;
  name: string;
  root_path: string;
  ecosystem: string | null;
  last_scanned: number | null;
  env_file_count: number;
}

export interface EnvFile {
  id: string;
  project_id: string;
  filename: string;
  relative_path: string;
  tier: string;
  depth: number;
  sub_variant: string | null;
  var_count: number;
  file_size: number;
  last_modified: number;
}

export interface EnvVar {
  id: string;
  file_id: string;
  key: string;
  encrypted_value: number[];
  nonce: number[];
  comment: string | null;
  line_number: number;
}

export interface ScanResult {
  root_path: string;
  projects_found: number;
  env_files_found: number;
  projects: ScanProject[];
}

export interface ScanProject {
  id: string;
  name: string;
  path: string;
  ecosystem: string | null;
  env_file_count: number;
}

export interface SearchResult {
  project_name: string;
  project_id: string;
  file_name: string;
  file_id: string;
  tier: string;
  key: string;
  var_id: string;
}

export interface ComparisonMatrix {
  files: ComparisonFile[];
  keys: ComparisonKey[];
}

export interface ComparisonFile {
  file_id: string;
  filename: string;
  tier: string;
}

export interface ComparisonKey {
  key: string;
  presence: boolean[];
  status: "all" | "some" | "single";
}

export interface SearchFilters {
  project_ids?: string[];
  tiers?: string[];
  status?: string[];
}

// ── API calls ────────────────────────────────────────────────────────

export const api = {
  // Roots
  addRoot: (path: string) => invoke<ScanResult>("add_root", { path }),
  removeRoot: (path: string) => invoke<void>("remove_root", { path }),
  scanRoot: (path: string) => invoke<ScanResult>("scan_root", { path }),
  scanAll: () => invoke<ScanResult[]>("scan_all"),
  getRoots: () => invoke<Root[]>("get_roots"),
  getProjects: () => invoke<Project[]>("get_projects"),

  // Env files
  getEnvFiles: (projectId: string) =>
    invoke<EnvFile[]>("get_env_files", { projectId }),
  getEnvVariables: (fileId: string) =>
    invoke<EnvVar[]>("get_env_variables", { fileId }),
  decryptValue: (encryptedValue: number[], nonce: number[]) =>
    invoke<string>("decrypt_value", { encryptedValue, nonce }),
  compareEnvs: (fileIds: string[]) =>
    invoke<ComparisonMatrix>("compare_envs", { fileIds }),

  // Vault
  setupVault: (masterPw: string) =>
    invoke<void>("setup_vault", { masterPw }),
  unlockVault: (masterPw: string) =>
    invoke<boolean>("unlock_vault", { masterPw }),
  lockVault: () => invoke<void>("lock_vault"),
  isVaultSetup: () => invoke<boolean>("is_vault_setup"),
  isVaultUnlocked: () => invoke<boolean>("is_vault_unlocked"),
  changePassword: (oldPw: string, newPw: string) =>
    invoke<void>("change_password", { oldPw, newPw }),
  exportAll: (targetDir: string) =>
    invoke<number>("export_all", { targetDir }),

  // Search
  search: (query: string, filters?: SearchFilters) =>
    invoke<SearchResult[]>("search", { query, filters }),

  // Watcher
  startWatcher: () => invoke<void>("start_watcher"),
  stopWatcher: () => invoke<void>("stop_watcher"),
};
