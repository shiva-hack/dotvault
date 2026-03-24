import { useState } from "react";

interface UnlockScreenProps {
  onUnlock: (password: string) => Promise<void>;
}

export function UnlockScreen({ onUnlock }: UnlockScreenProps) {
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setLoading(true);
    try {
      await onUnlock(password);
    } catch (e) {
      setError(String(e));
      setPassword("");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="h-screen bg-bg flex items-center justify-center">
      <div className="w-full max-w-sm">
        <div className="text-center mb-6">
          <div className="text-4xl mb-3">🔒</div>
          <h1 className="text-xl font-semibold text-white mb-1">Vault Locked</h1>
          <p className="text-muted text-sm">Enter your master password to unlock</p>
        </div>

        <form
          onSubmit={handleSubmit}
          className="bg-surface border border-border rounded-xl p-6 space-y-4"
        >
          <input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            placeholder="Master password"
            className="w-full px-3 py-2.5 bg-bg border border-border rounded-lg text-white text-sm outline-none focus:border-accent placeholder:text-muted"
            autoFocus
          />

          {error && <p className="text-red text-sm">{error}</p>}

          <button
            type="submit"
            disabled={loading}
            className="w-full py-2.5 bg-accent hover:bg-accent-hover text-white font-medium rounded-lg text-sm disabled:opacity-50"
          >
            {loading ? "Unlocking..." : "Unlock"}
          </button>
        </form>
      </div>
    </div>
  );
}
