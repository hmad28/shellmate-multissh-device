import { useEffect, useState, useCallback } from 'react';
import type { Update } from '@tauri-apps/plugin-updater';

interface UpdateInfo {
  version: string;
  date: string;
  body: string | null;
}

export function useAutoUpdater() {
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [updateRef, setUpdateRef] = useState<Update | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [readyToInstall, setReadyToInstall] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [dismissed, setDismissed] = useState(false);

  const checkForUpdates = useCallback(async () => {
    try {
      const { check } = await import('@tauri-apps/plugin-updater');
      const update = await check();
      if (update) {
        setUpdateRef(update);
        setUpdateInfo({
          version: update.version,
          date: update.date ?? '',
          body: update.body ?? null,
        });
      }
    } catch {
      // Updater not configured or network error — silently ignore
    }
  }, []);

  const downloadAndInstall = useCallback(async () => {
    if (!updateRef) return;
    setDownloading(true);
    setError(null);
    try {
      await updateRef.downloadAndInstall();
      setReadyToInstall(true);
    } catch (e) {
      setError(String(e));
    } finally {
      setDownloading(false);
    }
  }, [updateRef]);

  const relaunch = useCallback(async () => {
    try {
      // Relaunch by restarting the app via process restart
      const { exit } = await import('@tauri-apps/plugin-process');
      await exit(0);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    void checkForUpdates();
    const interval = setInterval(() => void checkForUpdates(), 30 * 60 * 1000);
    return () => clearInterval(interval);
  }, [checkForUpdates]);

  return {
    updateInfo,
    downloading,
    readyToInstall,
    error,
    dismissed,
    setDismissed,
    downloadAndInstall,
    relaunch,
    checkForUpdates,
  };
}
