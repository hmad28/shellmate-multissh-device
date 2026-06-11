import { useEffect, useRef, useState } from 'react';
import { Plug, Pencil, Trash2 } from 'lucide-react';
import { strings } from '@/i18n/en';
import { cn } from '@/lib/cn';
import { tauri } from '@/lib/tauri';
import { useHostStore } from '@/stores/host-store';
import { useSshStore } from '@/stores/ssh-store';
import { useTabStore } from '@/stores/tab-store';
import { ConfirmDialog } from '@/components/ui/ConfirmDialog';
import type { Host } from '@/types';

interface HostItemProps {
  host: Host;
  onEdit: () => void;
}

export function HostItem({ host, onEdit }: HostItemProps) {
  const groups = useHostStore((s) => s.groups);
  const deleteHost = useHostStore((s) => s.deleteHost);
  const addTab = useTabStore((s) => s.addTab);

  const [menu, setMenu] = useState<{ x: number; y: number } | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [connecting, setConnecting] = useState(false);

  const groupColor = host.groupId
    ? (groups.find((g) => g.id === host.groupId)?.color ?? null)
    : null;

  const handleConnect = async () => {
    if (connecting) return;
    setConnecting(true);
    const tabId = addTab({ label: host.label });
    try {
      await useSshStore.getState().connectSaved(tabId, host.id);
    } catch (err) {
      console.error('SSH connect failed', err);
    } finally {
      setConnecting(false);
    }
  };

  const handleDelete = async () => {
    setDeleting(true);
    try {
      // Best-effort: also delete the credential associated with this host
      try {
        await tauri.credentials.delete(host.credentialId);
      } catch {
        // credential might already be gone; ignore
      }
      await deleteHost(host.id);
      setConfirmDelete(false);
    } catch (err) {
      console.error('Failed to delete host', err);
    } finally {
      setDeleting(false);
    }
  };

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    setMenu({ x: e.clientX, y: e.clientY });
  };

  const handleDragStart = (e: React.DragEvent) => {
    e.dataTransfer.setData('application/x-shellmate-host', host.id);
    e.dataTransfer.effectAllowed = 'move';
  };

  return (
    <>
      <div
        role="button"
        tabIndex={0}
        draggable
        onDragStart={handleDragStart}
        onDoubleClick={handleConnect}
        onKeyDown={(e) => {
          if (e.key === 'Enter') {
            e.preventDefault();
            void handleConnect();
          }
        }}
        onContextMenu={handleContextMenu}
        className={cn(
          'group/host relative flex cursor-pointer items-center gap-2 rounded-md px-2 py-1 text-sm',
          'text-fg-muted transition-colors hover:bg-bg-elevated hover:text-fg',
        )}
        title={`${host.username}@${host.hostname}:${host.port}`}
      >
        {groupColor ? (
          <span
            className="inline-block h-1.5 w-1.5 shrink-0 rounded-full"
            style={{ backgroundColor: groupColor }}
            aria-hidden="true"
          />
        ) : (
          <span
            className="inline-block h-1.5 w-1.5 shrink-0"
            aria-hidden="true"
          />
        )}
        <span className="truncate">{host.label}</span>
        <span className="ml-auto truncate text-xs text-fg-subtle">
          {host.username}@{host.hostname}
        </span>
        <button
          type="button"
          onClick={(e) => {
            e.stopPropagation();
            void handleConnect();
          }}
          aria-label={`${strings.hostActions.connect} ${host.label}`}
          className={cn(
            'invisible flex h-5 w-5 items-center justify-center rounded',
            'text-fg-subtle hover:bg-border-strong hover:text-fg',
            'group-hover/host:visible',
          )}
        >
          <Plug size={11} />
        </button>
      </div>

      {menu && (
        <ContextMenu
          x={menu.x}
          y={menu.y}
          onClose={() => setMenu(null)}
          items={[
            {
              label: strings.hostActions.connect,
              icon: <Plug size={12} />,
              onClick: () => {
                setMenu(null);
                void handleConnect();
              },
            },
            {
              label: strings.hostActions.edit,
              icon: <Pencil size={12} />,
              onClick: () => {
                setMenu(null);
                onEdit();
              },
            },
            {
              label: strings.hostActions.delete,
              icon: <Trash2 size={12} />,
              variant: 'danger',
              onClick: () => {
                setMenu(null);
                setConfirmDelete(true);
              },
            },
          ]}
        />
      )}

      <ConfirmDialog
        open={confirmDelete}
        title={strings.hostForm.deleteConfirmTitle}
        body={strings.hostForm.deleteConfirmBody}
        confirmLabel={strings.hostForm.delete}
        variant="danger"
        loading={deleting}
        onConfirm={handleDelete}
        onCancel={() => setConfirmDelete(false)}
      />
    </>
  );
}

interface MenuItem {
  label: string;
  icon?: React.ReactNode;
  onClick: () => void;
  variant?: 'default' | 'danger';
}

function ContextMenu({
  x,
  y,
  onClose,
  items,
}: {
  x: number;
  y: number;
  onClose: () => void;
  items: MenuItem[];
}) {
  const ref = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const onClick = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) onClose();
    };
    const onEsc = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('mousedown', onClick);
    window.addEventListener('keydown', onEsc);
    return () => {
      window.removeEventListener('mousedown', onClick);
      window.removeEventListener('keydown', onEsc);
    };
  }, [onClose]);

  return (
    <div
      ref={ref}
      role="menu"
      style={{ left: x, top: y }}
      className="fixed z-50 min-w-[160px] rounded-md border border-border-strong bg-bg-elevated py-1 shadow-lg"
    >
      {items.map((item) => (
        <button
          key={item.label}
          type="button"
          role="menuitem"
          onClick={item.onClick}
          className={cn(
            'flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs',
            item.variant === 'danger'
              ? 'hover:bg-status-disconnected/10 text-status-disconnected'
              : 'text-fg hover:bg-bg-panel',
          )}
        >
          {item.icon}
          <span>{item.label}</span>
        </button>
      ))}
    </div>
  );
}
