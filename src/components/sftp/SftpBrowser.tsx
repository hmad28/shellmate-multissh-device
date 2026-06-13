import { useEffect } from 'react';
import {
  ArrowLeft,
  FolderUp,
  Home,
  RefreshCw,
  Upload,
  FolderPlus,
  Download,
  Trash2,
  ChevronRight,
  HardDrive,
  Loader2,
} from 'lucide-react';
import { useSftpStore } from '@/stores/sftp-store';
import { Button } from '@/components/ui/Button';
import { listen } from '@tauri-apps/api/event';
import type { SftpProgressEvent } from '@/types/sftp';
import { cn } from '@/lib/cn';

interface SftpBrowserProps {
  sessionId: string;
  onClose: () => void;
}

export function SftpBrowser({ sessionId, onClose }: SftpBrowserProps) {
  const {
    browsers,
    activeBrowserId,
    openBrowser,
    closeBrowser,
    listDirectory,
    removeFile,
    mkdir,
    updateTransfer,
    removeTransfer,
  } = useSftpStore();

  const browser = activeBrowserId ? browsers[activeBrowserId] : null;

  useEffect(() => {
    let sftpId: string | null = null;
    let unlistenFn: (() => void) | null = null;

    const init = async () => {
      sftpId = await openBrowser(sessionId);
      if (!sftpId) return;

      const unlisten = await listen<SftpProgressEvent>(
        `sftp:progress:${sftpId}`,
        (event) => {
          const { transferId, bytesTransferred, totalBytes, filename } =
            event.payload;
          const progress =
            totalBytes > 0 ? (bytesTransferred / totalBytes) * 100 : 0;
          updateTransfer({
            id: transferId,
            filename,
            bytesTransferred,
            totalBytes,
            progress,
          });

          if (bytesTransferred >= totalBytes) {
            setTimeout(() => removeTransfer(transferId), 1000);
          }
        },
      );
      unlistenFn = unlisten;
    };

    init();

    return () => {
      unlistenFn?.();
      if (sftpId) closeBrowser(sftpId);
    };
  }, [sessionId]);

  if (!browser) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <div className="flex items-center gap-2 text-sm text-fg-muted">
          <Loader2 className="h-4 w-4 animate-spin" />
          Opening SFTP browser...
        </div>
      </div>
    );
  }

  const handleNavigate = (path: string) => {
    listDirectory(browser.sftpId, path);
  };

  const handleGoUp = () => {
    const parts = browser.currentPath.split('/').filter(Boolean);
    if (parts.length <= 1) {
      handleNavigate('.');
    } else {
      handleNavigate(parts.slice(0, -1).join('/'));
    }
  };

  const handleGoHome = () => {
    handleNavigate('.');
  };

  const handleUpload = () => {
    alert('Upload functionality requires file picker integration');
  };

  const handleDownload = async (_filename: string) => {
    alert('Download functionality requires save dialog integration');
  };

  const handleDelete = async (path: string) => {
    if (confirm(`Delete ${path}?`)) {
      await removeFile(browser.sftpId, path);
    }
  };

  const handleMkdir = async () => {
    const name = prompt('New directory name:');
    if (name) {
      const path =
        browser.currentPath === '.' ? name : `${browser.currentPath}/${name}`;
      await mkdir(browser.sftpId, path);
    }
  };

  const breadcrumbs = browser.currentPath.split('/').filter(Boolean);

  return (
    <div className="flex flex-1 flex-col overflow-hidden bg-bg">
      {/* Header bar */}
      <div className="flex items-center gap-2 border-b border-border px-3 py-2">
        {/* Back button */}
        <button
          onClick={onClose}
          className={cn(
            'flex h-8 w-8 items-center justify-center rounded-md',
            'text-fg-muted transition-colors',
            'hover:bg-bg-elevated hover:text-fg',
          )}
          title="Back to terminal (Esc)"
        >
          <ArrowLeft size={16} />
        </button>

        {/* Navigation buttons */}
        <div className="flex items-center gap-1">
          <button
            onClick={handleGoUp}
            disabled={browser.currentPath === '.'}
            className={cn(
              'flex h-8 w-8 items-center justify-center rounded-md',
              'text-fg-muted transition-colors',
              'hover:bg-bg-elevated hover:text-fg',
              'disabled:cursor-not-allowed disabled:opacity-30',
            )}
            title="Go up"
          >
            <FolderUp size={16} />
          </button>
          <button
            onClick={handleGoHome}
            className={cn(
              'flex h-8 w-8 items-center justify-center rounded-md',
              'text-fg-muted transition-colors',
              'hover:bg-bg-elevated hover:text-fg',
            )}
            title="Go to root"
          >
            <Home size={16} />
          </button>
          <button
            onClick={() => handleNavigate(browser.currentPath)}
            className={cn(
              'flex h-8 w-8 items-center justify-center rounded-md',
              'text-fg-muted transition-colors',
              'hover:bg-bg-elevated hover:text-fg',
            )}
            title="Refresh"
          >
            <RefreshCw size={16} />
          </button>
        </div>

        {/* Breadcrumb path */}
        <div className="flex flex-1 items-center gap-0.5 overflow-x-auto text-sm">
          <button
            onClick={handleGoHome}
            className="flex shrink-0 items-center gap-1 rounded px-1.5 py-0.5 text-fg-muted hover:bg-bg-elevated hover:text-fg"
          >
            <HardDrive size={14} />
            <span>/</span>
          </button>
          {breadcrumbs.map((crumb, idx) => {
            const path = breadcrumbs.slice(0, idx + 1).join('/');
            const isLast = idx === breadcrumbs.length - 1;
            return (
              <div key={idx} className="flex shrink-0 items-center">
                <ChevronRight size={12} className="text-fg-subtle" />
                <button
                  onClick={() => handleNavigate(path)}
                  className={cn(
                    'rounded px-1.5 py-0.5 transition-colors',
                    isLast
                      ? 'font-medium text-fg'
                      : 'text-fg-muted hover:bg-bg-elevated hover:text-fg',
                  )}
                >
                  {crumb}
                </button>
              </div>
            );
          })}
        </div>

        {/* Action buttons */}
        <div className="flex items-center gap-1">
          <Button variant="ghost" size="sm" onClick={handleUpload}>
            <Upload size={14} />
            Upload
          </Button>
          <Button variant="ghost" size="sm" onClick={handleMkdir}>
            <FolderPlus size={14} />
            New Folder
          </Button>
        </div>
      </div>

      {/* File List */}
      <div className="flex-1 overflow-auto">
        {browser.loading ? (
          <div className="flex h-32 items-center justify-center">
            <Loader2 className="h-4 w-4 animate-spin text-fg-muted" />
          </div>
        ) : browser.error ? (
          <div className="flex h-32 items-center justify-center">
            <div className="text-destructive text-sm">{browser.error}</div>
          </div>
        ) : browser.files.length === 0 ? (
          <div className="flex h-32 flex-col items-center justify-center gap-2">
            <div className="text-sm text-fg-muted">Empty directory</div>
          </div>
        ) : (
          <table className="w-full text-sm">
            <thead className="sticky top-0 bg-bg-sidebar">
              <tr>
                <th className="px-4 py-2 text-left font-medium text-fg-muted">
                  Name
                </th>
                <th className="w-28 px-4 py-2 text-right font-medium text-fg-muted">
                  Size
                </th>
                <th className="w-48 px-4 py-2 text-left font-medium text-fg-muted">
                  Modified
                </th>
                <th className="w-32 px-4 py-2 text-right font-medium text-fg-muted">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody>
              {browser.files.map((file) => {
                const path =
                  browser.currentPath === '.'
                    ? file.name
                    : `${browser.currentPath}/${file.name}`;
                const modified = new Date(
                  file.modified * 1000,
                ).toLocaleString();
                const size = file.isDir ? '-' : formatBytes(file.size);

                return (
                  <tr
                    key={file.name}
                    className={cn(
                      'border-t border-border-subtle transition-colors',
                      'hover:bg-bg-elevated',
                    )}
                  >
                    <td className="px-4 py-2">
                      <button
                        onClick={() => file.isDir && handleNavigate(path)}
                        className={cn(
                          'flex items-center gap-2',
                          file.isDir
                            ? 'cursor-pointer text-accent hover:underline'
                            : 'cursor-default text-fg',
                        )}
                        disabled={!file.isDir}
                      >
                        <span className="text-base">
                          {file.isDir ? '📁' : file.isSymlink ? '🔗' : '📄'}
                        </span>
                        <span>{file.name}</span>
                      </button>
                    </td>
                    <td className="px-4 py-2 text-right font-mono text-xs text-fg-muted">
                      {size}
                    </td>
                    <td className="px-4 py-2 text-xs text-fg-muted">
                      {modified}
                    </td>
                    <td className="px-4 py-2 text-right">
                      <div className="flex items-center justify-end gap-1">
                        {!file.isDir && (
                          <button
                            onClick={() => handleDownload(file.name)}
                            className="flex h-7 w-7 items-center justify-center rounded text-fg-muted hover:bg-bg-elevated hover:text-fg"
                            title="Download"
                          >
                            <Download size={14} />
                          </button>
                        )}
                        <button
                          onClick={() => handleDelete(path)}
                          className="hover:bg-destructive/10 hover:text-destructive flex h-7 w-7 items-center justify-center rounded text-fg-muted"
                          title="Delete"
                        >
                          <Trash2 size={14} />
                        </button>
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        )}
      </div>

      {/* Transfers */}
      {browser.transfers.length > 0 && (
        <div className="border-t border-border bg-bg-sidebar px-4 py-2">
          <div className="mb-2 text-xs font-medium text-fg-muted">
            Transfers
          </div>
          {browser.transfers.map((transfer) => (
            <div key={transfer.id} className="mb-2 last:mb-0">
              <div className="mb-1 flex items-center justify-between text-xs">
                <span className="truncate text-fg">{transfer.filename}</span>
                <span className="text-fg-muted">
                  {transfer.progress.toFixed(0)}%
                </span>
              </div>
              <div className="h-1.5 w-full rounded-full bg-border">
                <div
                  className="h-1.5 rounded-full bg-accent transition-all"
                  style={{ width: `${transfer.progress}%` }}
                />
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}
