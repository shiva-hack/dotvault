import { useState } from "react";

interface SetupScreenProps {
  onSetup: (password: string) => Promise<void>;
}

export function SetupScreen({ onSetup }: SetupScreenProps) {
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");

    if (password.length < 8) {
      setError("Password must be at least 8 characters");
      return;
    }
    if (password !== confirm) {
      setError("Passwords do not match");
      return;
    }

    setLoading(true);
    try {
      await onSetup(password);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="h-screen bg-bg flex items-center justify-center">
      <div className="w-full max-w-md">
        {/* Header */}
        <div className="text-center mb-8">
          <div className="text-5xl mb-4">🔐</div>
          <h1 className="text-2xl font-semibold text-white mb-2">dotvault</h1>
          <p className="text-muted text-sm">
            One vault for every secret, across every project, every environment.
          </p>
        </div>

        {/* Setup form */}
        <form
          onSubmit={handleSubmit}
          className="bg-surface border border-border rounded-xl p-6 space-y-4"
        >
          <div>
            <h2 className="text-lg font-medium text-white mb-1">Create Master Password</h2>
            <p className="text-sm text-muted">
              This password encrypts all your env file contents. Choose something strong — it's
              never stored anywhere.
            </p>
          </div>

          <div>
            <label className="block text-sm text-zinc-400 mb-1.5">Master Password</label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Enter master password"
              className="w-full px-3 py-2 bg-bg border border-border rounded-lg text-white text-sm outline-none focus:border-accent placeholder:text-muted"
              autoFocus
            />
          </div>

          <div>
            <label className="block text-sm text-zinc-400 mb-1.5">Confirm Password</label>
            <input
              type="password"
              value={confirm}
              onChange={(e) => setConfirm(e.target.value)}
              placeholder="Confirm master password"
              className="w-full px-3 py-2 bg-bg border border-border rounded-lg text-white text-sm outline-none focus:border-accent placeholder:text-muted"
            />
          </div>

          {error && (
            <p className="text-red text-sm">{error}</p>
          )}

          <button
            type="submit"
            disabled={loading}
            className="w-full py-2.5 bg-accent hover:bg-accent-hover text-white font-medium rounded-lg text-sm disabled:opacity-50"
          >
            {loading ? "Setting up vault..." : "Create Vault"}
          </button>
        </form>

        <p className="text-center text-xs text-muted/50 mt-4">
          Your secrets are encrypted with AES-256-GCM. The master password is derived via Argon2id.
        </p>
      </div>
    </div>
  );
}
