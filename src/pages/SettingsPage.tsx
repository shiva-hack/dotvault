import { useState } from "react";
import { api, type Root } from "../lib/tauri";
import { open } from "@tauri-apps/plugin-dialog";

interface SettingsPageProps {
  onLock: () => void;
  roots: Root[];
  onRefresh: () => void;
}

export function SettingsPage({ onLock, roots, onRefresh }: SettingsPageProps) {
  const [changePwOpen, setChangePwOpen] = useState(false);
  const [oldPw, setOldPw] = useState("");
  const [newPw, setNewPw] = useState("");
  const [confirmPw, setConfirmPw] = useState("");
  const [pwError, setPwError] = useState("");
  const [pwSuccess, setPwSuccess] = useState(false);
  const [exporting, setExporting] = useState(false);

  const handleChangePassword = async (e: React.FormEvent) => {
    e.preventDefault();
    setPwError("");
    setPwSuccess(false);

    if (newPw.length < 8) {
      setPwError("New password must be at least 8 characters");
      return;
    }
    if (newPw !== confirmPw) {
      setPwError("New passwords do not match");
      return;
    }

    try {
      await api.changePassword(oldPw, newPw);
      setPwSuccess(true);
      setOldPw("");
      setNewPw("");
      setConfirmPw("");
      setChangePwOpen(false);
    } catch (e) {
      setPwError(String(e));
    }
  };

  const handleExport = async () => {
    const selected = await open({ directory: true, multiple: false });
    if (!selected) return;

    setExporting(true);
    try {
      const count = await api.exportAll(selected as string);
      alert(`Exported ${count} env file(s) to ${selected}`);
    } catch (e) {
      alert(`Export failed: ${e}`);
    } finally {
      setExporting(false);
    }
  };

  return (
    <div className="max-w-2xl">
      <h1 className="text-xl font-semibold text-white mb-6">Settings</h1>

      {/* Vault section */}
      <section className="bg-surface border border-border rounded-lg p-5 mb-4">
        <h2 className="text-sm font-medium text-white mb-3">Vault</h2>

        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-zinc-300">Lock Vault</p>
              <p className="text-xs text-muted">Clears the encryption key from memory</p>
            </div>
            <button
              onClick={onLock}
              className="px-4 py-1.5 bg-surface-2 hover:bg-border rounded text-sm text-zinc-300"
            >
              Lock Now
            </button>
          </div>

          <div className="border-t border-border/50 pt-3 flex items-center justify-between">
            <div>
              <p className="text-sm text-zinc-300">Change Master Password</p>
              <p className="text-xs text-muted">Re-encrypts all stored data with a new key</p>
            </div>
            <button
              onClick={() => setChangePwOpen(!changePwOpen)}
              className="px-4 py-1.5 bg-surface-2 hover:bg-border rounded text-sm text-zinc-300"
            >
              {changePwOpen ? "Cancel" : "Change"}
            </button>
          </div>

          {changePwOpen && (
            <form onSubmit={handleChangePassword} className="space-y-3 pt-2">
              <input
                type="password"
                value={oldPw}
                onChange={(e) => setOldPw(e.target.value)}
                placeholder="Current password"
                className="w-full px-3 py-2 bg-bg border border-border rounded-lg text-white text-sm outline-none focus:border-accent placeholder:text-muted"
              />
              <input
                type="password"
                value={newPw}
                onChange={(e) => setNewPw(e.target.value)}
                placeholder="New password"
                className="w-full px-3 py-2 bg-bg border border-border rounded-lg text-white text-sm outline-none focus:border-accent placeholder:text-muted"
              />
              <input
                type="password"
                value={confirmPw}
                onChange={(e) => setConfirmPw(e.target.value)}
                placeholder="Confirm new password"
                className="w-full px-3 py-2 bg-bg border border-border rounded-lg text-white text-sm outline-none focus:border-accent placeholder:text-muted"
              />
              {pwError && <p className="text-red text-sm">{pwError}</p>}
              {pwSuccess && <p className="text-green text-sm">Password changed successfully</p>}
              <button
                type="submit"
                className="px-4 py-2 bg-accent hover:bg-accent-hover text-white rounded-lg text-sm"
              >
                Update Password
              </button>
            </form>
          )}
        </div>
      </section>

      {/* Roots section */}
      <section className="bg-surface border border-border rounded-lg p-5 mb-4">
        <h2 className="text-sm font-medium text-white mb-3">Root Directories</h2>

        {roots.length === 0 ? (
          <p className="text-sm text-muted">No root directories configured</p>
        ) : (
          <div className="space-y-2">
            {roots.map((root) => (
              <div
                key={root.path}
                className="flex items-center justify-between py-2 px-3 bg-bg rounded-lg"
              >
                <span className="text-sm font-mono text-zinc-300 truncate">{root.path}</span>
                <button
                  onClick={async () => {
                    await api.removeRoot(root.path);
                    onRefresh();
                  }}
                  className="text-xs text-red/60 hover:text-red ml-3 shrink-0"
                >
                  Remove
                </button>
              </div>
            ))}
          </div>
        )}
      </section>

      {/* Export section */}
      <section className="bg-surface border border-border rounded-lg p-5 mb-4">
        <h2 className="text-sm font-medium text-white mb-3">Emergency Export</h2>
        <p className="text-xs text-muted mb-3">
          Decrypt and export all stored env files back to a chosen directory as plaintext.
        </p>
        <button
          onClick={handleExport}
          disabled={exporting}
          className="px-4 py-2 bg-red/10 border border-red/30 hover:bg-red/20 text-red rounded-lg text-sm disabled:opacity-50"
        >
          {exporting ? "Exporting..." : "Export All (Plaintext)"}
        </button>
      </section>

      {/* About */}
      <section className="bg-surface border border-border rounded-lg p-5">
        <h2 className="text-sm font-medium text-white mb-2">About</h2>
        <p className="text-xs text-muted">
          dotvault v1.0.0 — One vault for every secret, across every project, every environment.
        </p>
        <p className="text-xs text-muted mt-1">
          Built with Tauri 2 + React + Rust. Encryption: AES-256-GCM + Argon2id.
        </p>
      </section>
    </div>
  );
}
