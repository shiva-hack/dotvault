import { useState, useEffect, useCallback } from "react";
import { api, type Root, type Project } from "../lib/tauri";

export function useVault() {
  const [isSetup, setIsSetup] = useState<boolean | null>(null);
  const [isUnlocked, setIsUnlocked] = useState(false);
  const [roots, setRoots] = useState<Root[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const checkStatus = useCallback(async () => {
    try {
      const setup = await api.isVaultSetup();
      setIsSetup(setup);
      if (setup) {
        const unlocked = await api.isVaultUnlocked();
        setIsUnlocked(unlocked);
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  const refresh = useCallback(async () => {
    try {
      const [rootList, projectList] = await Promise.all([
        api.getRoots(),
        api.getProjects(),
      ]);
      setRoots(rootList);
      setProjects(projectList);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    checkStatus();
  }, [checkStatus]);

  useEffect(() => {
    if (isUnlocked) {
      refresh();
    }
  }, [isUnlocked, refresh]);

  const setupVault = async (password: string) => {
    await api.setupVault(password);
    setIsSetup(true);
    setIsUnlocked(true);
  };

  const unlock = async (password: string) => {
    const ok = await api.unlockVault(password);
    if (!ok) throw new Error("Invalid password");
    setIsUnlocked(true);
  };

  const lock = async () => {
    await api.lockVault();
    setIsUnlocked(false);
  };

  return {
    isSetup,
    isUnlocked,
    roots,
    projects,
    loading,
    error,
    setupVault,
    unlock,
    lock,
    refresh,
    setError,
  };
}
