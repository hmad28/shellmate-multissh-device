import { useEffect } from 'react';
import { useSftpStore } from '@/stores/sftp-store';
import { Button } from '@/components/ui/Button';
import { listen } from '@tauri-apps/api/event';
import type { SftpProgressEvent } from '@/types/sftp';

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

    const init = async () => {
      sftpId = await openBrowser(sessionId);
    };

    init();

    const unlisten = listen<SftpProgressEvent>(`sftp:progress:${sftpId}`, (event) => {
      const { transferId, bytesTransferred, totalBytes, filename } = event.payload;
      const progress = totalBytes > 0 ? (bytesTransferred / totalBytes) * 100 : 0;
      updateTransfer({ id: transferId, filename, bytesTransferred, totalBytes, progress });

      if (bytesTransferred >= totalBytes) {
        setTimeout(() => removeTransfer(transferId), 1000);
      }
    });

    return () => {
      unlisten.then(fn => fn());
      if (sftpId) closeBrowser(sftpId);
    };
  }, [sessionId]);

  if (!browser) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-sm text-muted-foreground">Opening SFTP browser...</div>
      </div>
    );
  }

  const handleNavigate = (path: string) => {
    listDirectory(browser.sftpId, path);
  };

  const handleUpload = () => {
    // TODO: Integrate with Tauri file picker dialog
    alert('Upload functionality requires file picker integration');
  };

  const handleDownload = async (_filename: string) => {
    // TODO: Integrate with Tauri save dialog
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
      const path = browser.currentPath === '.' ? name : `${browser.currentPath}/${name}`;
      await mkdir(browser.sftpId, path);
    }
  };

  const breadcrumbs = browser.currentPath.split('/').filter(Boolean);
  const currentPath = browser.currentPath === '.' ? '/' : browser.currentPath;

  return (
    <div className="flex flex-col h-full bg-background border-t">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2 border-b">
        <div className="flex items-center gap-2">
          <span className="font-semibold text-sm">SFTP Browser</span>
          <span className="text-xs text-muted-foreground">{currentPath}</span>
        </div>
        <Button variant="ghost" size="sm" onClick={onClose}>
          Close
        </Button>
      </div>

      {/* Breadcrumbs */}
      <div className="flex items-center gap-1 px-4 py-2 border-b bg-muted/50">
        <button
          onClick={() => handleNavigate('.')}
          className="text-xs hover:underline text-muted-foreground"
        >
          /
        </button>
        {breadcrumbs.map((crumb, idx) => {
          const path = breadcrumbs.slice(0, idx + 1).join('/');
          return (
            <span key={idx} className="flex items-center gap-1">
              <span className="text-xs text-muted-foreground">/</span>
              <button
                onClick={() => handleNavigate(path)}
                className="text-xs hover:underline"
              >
                {crumb}
              </button>
            </span>
          );
        })}
      </div>

      {/* Toolbar */}
      <div className="flex items-center gap-2 px-4 py-2 border-b">
        <Button variant="secondary" size="sm" onClick={handleUpload}>
          Upload
        </Button>
        <Button variant="secondary" size="sm" onClick={handleMkdir}>
          New Folder
        </Button>
        <Button
          variant="secondary"
          size="sm"
          onClick={() => handleNavigate(browser.currentPath)}
        >
          Refresh
        </Button>
      </div>

      {/* File List */}
      <div className="flex-1 overflow-auto">
        {browser.loading ? (
          <div className="flex items-center justify-center h-32">
            <div className="text-sm text-muted-foreground">Loading...</div>
          </div>
        ) : browser.error ? (
          <div className="flex items-center justify-center h-32">
            <div className="text-sm text-destructive">{browser.error}</div>
          </div>
        ) : browser.files.length === 0 ? (
          <div className="flex items-center justify-center h-32">
            <div className="text-sm text-muted-foreground">Empty directory</div>
          </div>
        ) : (
          <table className="w-full text-sm">
            <thead className="bg-muted/50 sticky top-0">
              <tr>
                <th className="text-left px-4 py-2 font-medium">Name</th>
                <th className="text-right px-4 py-2 font-medium">Size</th>
                <th className="text-left px-4 py-2 font-medium">Modified</th>
                <th className="text-right px-4 py-2 font-medium">Actions</th>
              </tr>
            </thead>
            <tbody>
              {browser.files.map((file) => {
                const path =
                  browser.currentPath === '.'
                    ? file.name
                    : `${browser.currentPath}/${file.name}`;
                const modified = new Date(file.modified * 1000).toLocaleString();
                const size = file.isDir ? '-' : formatBytes(file.size);

                return (
                  <tr key={file.name} className="border-t hover:bg-muted/50">
                    <td className="px-4 py-2">
                      <button
                        onClick={() => file.isDir && handleNavigate(path)}
                        className="flex items-center gap-2 hover:underline"
                        disabled={!file.isDir}
                      >
                        <span className="text-lg">
                          {file.isDir ? '📁' : file.isSymlink ? '🔗' : '📄'}
                        </span>
                        <span>{file.name}</span>
                      </button>
                    </td>
                    <td className="px-4 py-2 text-right text-muted-foreground">{size}</td>
                    <td className="px-4 py-2 text-muted-foreground">{modified}</td>
                    <td className="px-4 py-2 text-right">
                      <div className="flex items-center justify-end gap-1">
                        {!file.isDir && (
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => handleDownload(file.name)}
                          >
                            Download
                          </Button>
                        )}
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleDelete(path)}
                        >
                          Delete
                        </Button>
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
        <div className="border-t bg-muted/30 px-4 py-2">
          <div className="text-xs font-medium mb-2">Transfers</div>
          {browser.transfers.map((transfer) => (
            <div key={transfer.id} className="mb-2 last:mb-0">
              <div className="flex items-center justify-between text-xs mb-1">
                <span className="truncate">{transfer.filename}</span>
                <span className="text-muted-foreground">
                  {transfer.progress.toFixed(0)}%
                </span>
              </div>
              <div className="w-full bg-muted rounded-full h-1.5">
                <div
                  className="bg-primary h-1.5 rounded-full transition-all"
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
