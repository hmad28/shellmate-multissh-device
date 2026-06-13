import { Download, RefreshCw, X, Loader2 } from 'lucide-react';
import { useAutoUpdater } from '@/hooks/useAutoUpdater';
import { cn } from '@/lib/cn';

export function UpdateToast() {
  const {
    updateInfo,
    downloading,
    readyToInstall,
    error,
    dismissed,
    setDismissed,
    downloadAndInstall,
    relaunch,
  } = useAutoUpdater();

  if (!updateInfo || dismissed) return null;

  return (
    <div
      className={cn(
        'fixed bottom-10 right-4 z-50 w-80 overflow-hidden rounded-lg border border-border bg-bg shadow-xl',
        'animate-in slide-in-from-bottom-4',
      )}
    >
      <div className="flex items-center justify-between border-b border-border px-4 py-2">
        <span className="text-xs font-medium text-fg">Update Available</span>
        <button
          onClick={() => setDismissed(true)}
          className="text-fg-muted hover:text-fg"
        >
          <X size={14} />
        </button>
      </div>

      <div className="p-4">
        <p className="text-sm text-fg">
          ShellMate <span className="font-semibold">v{updateInfo.version}</span> is available.
        </p>
        {updateInfo.body && (
          <p className="mt-1 max-h-20 overflow-auto text-xs text-fg-muted">
            {updateInfo.body}
          </p>
        )}

        {error && (
          <p className="mt-2 text-xs text-red-400">{error}</p>
        )}

        <div className="mt-3 flex gap-2">
          {readyToInstall ? (
            <button
              onClick={() => void relaunch()}
              className="flex flex-1 items-center justify-center gap-2 rounded-md bg-accent px-3 py-1.5 text-xs font-medium text-white hover:bg-accent-hover"
            >
              <RefreshCw size={12} />
              Restart Now
            </button>
          ) : (
            <button
              onClick={() => void downloadAndInstall()}
              disabled={downloading}
              className="flex flex-1 items-center justify-center gap-2 rounded-md bg-accent px-3 py-1.5 text-xs font-medium text-white hover:bg-accent-hover disabled:opacity-50"
            >
              {downloading ? (
                <Loader2 size={12} className="animate-spin" />
              ) : (
                <Download size={12} />
              )}
              {downloading ? 'Downloading...' : 'Download & Install'}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
