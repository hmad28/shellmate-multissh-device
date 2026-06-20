import React, { useState, useEffect } from 'react';
import { tauri } from '@/lib/tauri';
import {
  Folder,
  File,
  ArrowUp,
  RefreshCw,
  Download,
  Trash2,
  Upload,
  AlertCircle,
  Loader2,
  Home,
} from 'lucide-react';

interface FileItem {
  name: string;
  size: number;
  isDir: boolean;
  modified?: number;
}

export function FileExplorerTab() {
  const [currentPath, setCurrentPath] = useState<string>('');
  const [items, setItems] = useState<FileItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [addressInput, setAddressInput] = useState<string>('');

  const loadDirectory = async (path?: string) => {
    setLoading(true);
    setError(null);
    try {
      const res = await tauri.p2p.listRemoteFiles(path);
      if (res.success) {
        setCurrentPath(res.currentPath);
        setAddressInput(res.currentPath);
        
        // Sort: directories first, then files (alphabetically)
        const sortedItems = [...res.items].sort((a, b) => {
          if (a.isDir && !b.isDir) return -1;
          if (!a.isDir && b.isDir) return 1;
          return a.name.localeCompare(b.name);
        });
        setItems(sortedItems);
      } else {
        setError('Failed to fetch file list');
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void loadDirectory();
  }, []);

  const handleNavigate = (folderName: string) => {
    const sep = currentPath.includes('\\') ? '\\' : '/';
    const newPath = currentPath.endsWith(sep)
      ? `${currentPath}${folderName}`
      : `${currentPath}${sep}${folderName}`;
    void loadDirectory(newPath);
  };

  const handleGoUp = () => {
    const isWindows = currentPath.includes('\\');
    const sep = isWindows ? '\\' : '/';
    const parts = currentPath.split(sep).filter(Boolean);
    
    if (parts.length <= 1) {
      // If we are at the root drive/folder
      if (isWindows && currentPath.length <= 3) return; // e.g. C:\
      void loadDirectory(isWindows ? 'C:\\' : '/');
    } else {
      let parent = parts.slice(0, -1).join(sep);
      if (isWindows && !parent.endsWith(':') && parts[0]?.endsWith(':')) {
        // preserve drive letter syntax
        parent = parent;
      }
      if (!isWindows) {
        parent = '/' + parent;
      }
      void loadDirectory(parent);
    }
  };

  const handleAddressSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (addressInput.trim()) {
      void loadDirectory(addressInput.trim());
    }
  };

  const handleDelete = async (itemName: string) => {
    const sep = currentPath.includes('\\') ? '\\' : '/';
    const targetPath = currentPath.endsWith(sep)
      ? `${currentPath}${itemName}`
      : `${currentPath}${sep}${itemName}`;

    if (confirm(`Are you sure you want to delete "${itemName}"?`)) {
      setLoading(true);
      try {
        const res = await tauri.p2p.deleteRemoteFile(targetPath);
        if (res.success) {
          void loadDirectory(currentPath);
        } else {
          setError('Failed to delete file');
        }
      } catch (e) {
        setError(String(e));
      } finally {
        setLoading(false);
      }
    }
  };

  const handleDownload = async (itemName: string) => {
    const sep = currentPath.includes('\\') ? '\\' : '/';
    const remotePath = currentPath.endsWith(sep)
      ? `${currentPath}${itemName}`
      : `${currentPath}${sep}${itemName}`;

    const localPath = prompt(
      `Enter local absolute path to save "${itemName}":`,
      `C:\\Users\\Pongo\\Downloads\\${itemName}`,
    );

    if (localPath) {
      setLoading(true);
      setError(null);
      try {
        await tauri.p2p.downloadRemoteFile(remotePath, localPath);
        alert('File downloaded successfully!');
      } catch (e) {
        setError(String(e));
      } finally {
        setLoading(false);
      }
    }
  };

  const handleUpload = async () => {
    const localPath = prompt('Enter absolute path of local file to upload:');
    if (!localPath) return;

    // Extract filename from local path
    const localFilename = localPath.split(/[\\/]/).pop() || 'uploaded_file';
    const sep = currentPath.includes('\\') ? '\\' : '/';
    const remotePath = currentPath.endsWith(sep)
      ? `${currentPath}${localFilename}`
      : `${currentPath}${sep}${localFilename}`;

    setLoading(true);
    setError(null);
    try {
      await tauri.p2p.uploadRemoteFile(remotePath, localPath);
      alert('File uploaded successfully!');
      void loadDirectory(currentPath);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const formatSize = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  };

  const isNoDeviceConfigured = error?.toLowerCase().includes('no paired desktop');

  return (
    <div className="flex flex-col h-[500px] bg-bg rounded-lg overflow-hidden border border-border">
      {/* Address Bar & Operations Toolbar */}
      <div className="flex flex-col gap-2 p-3 border-b border-border bg-bg-panel text-xs">
        <div className="flex items-center gap-2">
          {/* Navigation Controls */}
          <button
            onClick={handleGoUp}
            disabled={loading || !currentPath || isNoDeviceConfigured}
            className="p-1.5 rounded border border-border bg-bg-sidebar hover:bg-bg-elevated text-fg disabled:opacity-50"
            title="Go to Parent Folder"
          >
            <ArrowUp className="h-4 w-4" />
          </button>
          
          <button
            onClick={() => void loadDirectory()}
            disabled={loading || isNoDeviceConfigured}
            className="p-1.5 rounded border border-border bg-bg-sidebar hover:bg-bg-elevated text-fg disabled:opacity-50"
            title="Go to Home Folder"
          >
            <Home className="h-4 w-4" />
          </button>

          {/* Address Bar Form */}
          <form onSubmit={handleAddressSubmit} className="flex-1 flex gap-1">
            <input
              type="text"
              value={addressInput}
              onChange={(e) => setAddressInput(e.target.value)}
              disabled={loading || isNoDeviceConfigured}
              className="flex-1 rounded border border-border-subtle bg-bg px-3 py-1.5 font-mono text-xs text-fg outline-none focus:border-accent disabled:opacity-50"
              placeholder="Remote Directory Path (e.g. C:\Users\Pongo or /home/user)"
            />
            <button
              type="submit"
              disabled={loading || isNoDeviceConfigured}
              className="px-3 py-1.5 rounded bg-accent hover:bg-accent-hover font-semibold text-white disabled:opacity-50"
            >
              Go
            </button>
          </form>

          {/* Refresh & Upload */}
          <button
            onClick={() => void loadDirectory(currentPath)}
            disabled={loading || isNoDeviceConfigured}
            className="p-1.5 rounded border border-border bg-bg-sidebar hover:bg-bg-elevated text-fg disabled:opacity-50"
            title="Refresh"
          >
            {loading ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <RefreshCw className="h-4 w-4" />
            )}
          </button>

          <button
            onClick={handleUpload}
            disabled={loading || isNoDeviceConfigured}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded bg-accent hover:bg-accent-hover font-semibold text-white disabled:opacity-50"
            title="Upload File"
          >
            <Upload className="h-3.5 w-3.5" />
            <span>Upload</span>
          </button>
        </div>
      </div>

      {/* Directory Contents List */}
      <div className="flex-1 overflow-y-auto bg-bg p-1">
        {items.length > 0 ? (
          <table className="w-full text-left border-collapse text-xs select-none">
            <thead>
              <tr className="border-b border-border bg-bg-sidebar text-fg-muted font-medium">
                <th className="p-2.5 pl-3">Name</th>
                <th className="p-2.5 w-24">Size</th>
                <th className="p-2.5 w-40">Modified</th>
                <th className="p-2.5 w-20 text-right pr-3">Actions</th>
              </tr>
            </thead>
            <tbody>
              {items.map((item) => (
                <tr
                  key={item.name}
                  className="border-b border-border-subtle hover:bg-bg-elevated/40 transition-all text-fg"
                >
                  <td className="p-2.5 pl-3 font-medium">
                    {item.isDir ? (
                      <button
                        onClick={() => handleNavigate(item.name)}
                        className="flex items-center gap-2 text-left hover:text-accent font-semibold outline-none"
                      >
                        <Folder className="h-4 w-4 text-yellow-500 shrink-0" />
                        <span>{item.name}</span>
                      </button>
                    ) : (
                      <div className="flex items-center gap-2">
                        <File className="h-4 w-4 text-blue-400 shrink-0" />
                        <span>{item.name}</span>
                      </div>
                    )}
                  </td>
                  <td className="p-2.5 font-mono text-[11px] text-fg-muted">
                    {item.isDir ? '—' : formatSize(item.size)}
                  </td>
                  <td className="p-2.5 text-fg-muted">
                    {item.modified
                      ? new Date(item.modified * 1000).toLocaleString()
                      : '—'}
                  </td>
                  <td className="p-2.5 text-right pr-3">
                    <div className="inline-flex items-center gap-1.5">
                      {!item.isDir && (
                        <button
                          onClick={() => handleDownload(item.name)}
                          disabled={loading}
                          className="p-1 rounded text-fg-muted hover:bg-accent/10 hover:text-accent disabled:opacity-50"
                          title="Download file"
                        >
                          <Download className="h-3.5 w-3.5" />
                        </button>
                      )}
                      <button
                        onClick={() => handleDelete(item.name)}
                        disabled={loading}
                        className="p-1 rounded text-fg-muted hover:bg-red-500/10 hover:text-red-400 disabled:opacity-50"
                        title="Delete path"
                      >
                        <Trash2 className="h-3.5 w-3.5" />
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          !loading && (
            <div className="h-full flex flex-col items-center justify-center text-center p-6 space-y-3">
              <Folder className="h-12 w-12 text-fg-muted animate-pulse" />
              <div className="text-sm font-semibold text-fg">Empty Folder</div>
              <p className="text-xs text-fg-muted max-w-xs mx-auto">
                No items were found in this directory, or the remote host has not connected yet.
              </p>
            </div>
          )
        )}

        {/* Loading overlay */}
        {loading && (
          <div className="h-full flex items-center justify-center">
            <div className="text-center space-y-2">
              <Loader2 className="h-8 w-8 text-accent animate-spin mx-auto" />
              <div className="text-xs text-fg-muted">Syncing remote files...</div>
            </div>
          </div>
        )}

        {/* Error / pairing warning banner */}
        {error && (
          <div className="m-3 p-3 rounded-lg bg-red-500/10 border border-red-500/20 flex items-start gap-2.5">
            <AlertCircle className="h-4 w-4 text-red-500 shrink-0 mt-0.5" />
            <div className="text-[11px] leading-4 text-red-400">
              {isNoDeviceConfigured ? (
                <span>
                  <strong>No remote desktop paired.</strong> Please pair another desktop in the <strong>Pairing & Devices</strong> tab first to navigate files.
                </span>
              ) : (
                <span>Error loading directories: {error}</span>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
