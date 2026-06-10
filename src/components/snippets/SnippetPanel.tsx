import { useEffect, useMemo, useState } from 'react';
import { Code2, Pencil, Play, Plus, Search, Trash2 } from 'lucide-react';
import { strings } from '@/i18n/en';
import { Button } from '@/components/ui/Button';
import { ConfirmDialog } from '@/components/ui/ConfirmDialog';
import { cn } from '@/lib/cn';
import { expandSnippet, extractPlaceholders } from '@/lib/snippet-expand';
import { tauri } from '@/lib/tauri';
import { useHostStore } from '@/stores/host-store';
import { useSnippetStore } from '@/stores/snippet-store';
import { useSshStore } from '@/stores/ssh-store';
import { useTabStore } from '@/stores/tab-store';
import { SnippetForm } from './SnippetForm';
import type { Snippet } from '@/types/snippet';

export function SnippetPanel() {
  const { snippets, loaded, load, remove, searchQuery, setSearchQuery } =
    useSnippetStore();

  const tabs = useTabStore((s) => s.tabs);
  const activeTabId = useTabStore((s) => s.activeTabId);
  const sessionByTab = useSshStore((s) => s.sessionByTab);
  const hosts = useHostStore((s) => s.hosts);

  const [formOpen, setFormOpen] = useState(false);
  const [editing, setEditing] = useState<Snippet | null>(null);
  const [confirmDelete, setConfirmDelete] = useState<Snippet | null>(null);
  const [feedback, setFeedback] = useState<string | null>(null);

  useEffect(() => {
    if (!loaded) void load();
  }, [loaded, load]);

  const activeTab = tabs.find((t) => t.id === activeTabId) ?? null;
  const activeSessionId = activeTab
    ? (sessionByTab[activeTab.id] ?? null)
    : null;
  const activeHost = activeTab?.hostId
    ? (hosts.find((h) => h.id === activeTab.hostId) ?? null)
    : null;

  const filtered = useMemo(() => {
    const q = searchQuery.trim().toLowerCase();
    if (!q) return snippets;
    return snippets.filter((s) =>
      [s.title, s.command, s.description ?? '', ...s.tags]
        .join(' ')
        .toLowerCase()
        .includes(q),
    );
  }, [snippets, searchQuery]);

  const handleExecute = async (snippet: Snippet) => {
    if (!activeSessionId) {
      setFeedback(strings.snippets.noActiveSession);
      return;
    }
    const placeholders = extractPlaceholders(snippet.command);
    const known = new Set(['username', 'host', 'hostname', 'port', 'label']);
    const unknown = placeholders.filter((p) => !known.has(p));
    if (unknown.length > 0) {
      setFeedback(
        `${strings.snippets.placeholderWarning} (${unknown.join(', ')})`,
      );
      return;
    }
    const expanded = expandSnippet(snippet.command, { host: activeHost });
    const payload = expanded.endsWith('\n') ? expanded : `${expanded}\n`;
    try {
      await tauri.ssh.send(activeSessionId, payload);
      setFeedback(`${strings.snippets.executed}: ${snippet.title}`);
    } catch (err) {
      setFeedback(`${strings.snippets.failed}: ${String(err)}`);
    }
  };

  const handleEdit = (snippet: Snippet) => {
    setEditing(snippet);
    setFormOpen(true);
  };

  const handleAdd = () => {
    setEditing(null);
    setFormOpen(true);
  };

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center justify-between gap-3 border-b border-border-subtle px-4 py-3">
        <div className="flex items-center gap-2">
          <Code2 size={16} className="text-accent" aria-hidden="true" />
          <h1 className="text-sm font-semibold text-fg">
            {strings.snippets.heading}
          </h1>
        </div>
        <Button variant="primary" size="sm" onClick={handleAdd}>
          <Plus size={12} />
          <span>{strings.snippets.add}</span>
        </Button>
      </header>

      <div className="border-b border-border-subtle px-4 py-2">
        <div className="relative">
          <Search
            size={14}
            className="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-fg-subtle"
            aria-hidden="true"
          />
          <input
            type="search"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder={strings.snippets.searchPlaceholder}
            aria-label={strings.snippets.searchPlaceholder}
            className={cn(
              'w-full rounded-md bg-bg-elevated py-1.5 pl-8 pr-2 text-sm',
              'text-fg placeholder:text-fg-subtle',
              'border border-border-subtle',
              'focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent',
            )}
          />
        </div>
      </div>

      {feedback && (
        <div
          role="status"
          aria-live="polite"
          className="border-b border-border-subtle bg-bg-panel px-4 py-2 text-xs text-fg-muted"
        >
          {feedback}
        </div>
      )}

      <div className="flex-1 overflow-y-auto">
        {filtered.length === 0 ? (
          <EmptyState hasSnippets={snippets.length > 0} />
        ) : (
          <ul className="divide-y divide-border-subtle">
            {filtered.map((s) => (
              <li key={s.id}>
                <SnippetRow
                  snippet={s}
                  onExecute={() => handleExecute(s)}
                  onEdit={() => handleEdit(s)}
                  onDelete={() => setConfirmDelete(s)}
                  canExecute={!!activeSessionId}
                />
              </li>
            ))}
          </ul>
        )}
      </div>

      <SnippetForm
        open={formOpen}
        onClose={() => setFormOpen(false)}
        snippet={editing}
      />

      <ConfirmDialog
        open={!!confirmDelete}
        title={strings.snippets.deleteConfirmTitle}
        body={strings.snippets.deleteConfirmBody}
        confirmLabel={strings.snippets.delete}
        variant="danger"
        onConfirm={async () => {
          if (!confirmDelete) return;
          await remove(confirmDelete.id);
          setConfirmDelete(null);
        }}
        onCancel={() => setConfirmDelete(null)}
      />
    </div>
  );
}

function SnippetRow({
  snippet,
  onExecute,
  onEdit,
  onDelete,
  canExecute,
}: {
  snippet: Snippet;
  onExecute: () => void;
  onEdit: () => void;
  onDelete: () => void;
  canExecute: boolean;
}) {
  return (
    <div className="flex items-start gap-3 px-4 py-3 hover:bg-bg-panel">
      <div className="min-w-0 flex-1">
        <div className="flex items-baseline gap-2">
          <span className="truncate text-sm font-medium text-fg">
            {snippet.title}
          </span>
          {snippet.tags.map((t) => (
            <span
              key={t}
              className="rounded bg-bg-elevated px-1.5 py-0.5 text-[10px] text-fg-muted"
            >
              {t}
            </span>
          ))}
        </div>
        {snippet.description && (
          <p className="mt-0.5 truncate text-xs text-fg-muted">
            {snippet.description}
          </p>
        )}
        <pre className="mt-1.5 truncate rounded bg-bg-elevated px-2 py-1 font-mono text-xs text-fg">
          {snippet.command.split('\n')[0]}
          {snippet.command.includes('\n') ? ' …' : ''}
        </pre>
      </div>
      <div className="flex shrink-0 gap-1 pt-0.5">
        <button
          type="button"
          onClick={onExecute}
          disabled={!canExecute}
          aria-label={strings.snippets.execute}
          title={
            canExecute
              ? strings.snippets.execute
              : strings.snippets.noActiveSession
          }
          className={cn(
            'flex h-7 w-7 items-center justify-center rounded text-fg-muted hover:bg-bg-elevated hover:text-accent',
            !canExecute && 'cursor-not-allowed opacity-40 hover:bg-transparent',
          )}
        >
          <Play size={12} />
        </button>
        <button
          type="button"
          onClick={onEdit}
          aria-label={strings.snippets.edit}
          className="flex h-7 w-7 items-center justify-center rounded text-fg-muted hover:bg-bg-elevated hover:text-fg"
        >
          <Pencil size={12} />
        </button>
        <button
          type="button"
          onClick={onDelete}
          aria-label={strings.snippets.delete}
          className="hover:bg-status-disconnected/10 flex h-7 w-7 items-center justify-center rounded text-fg-muted hover:text-status-disconnected"
        >
          <Trash2 size={12} />
        </button>
      </div>
    </div>
  );
}

function EmptyState({ hasSnippets }: { hasSnippets: boolean }) {
  return (
    <div className="flex flex-col items-center gap-3 p-12 text-center text-sm text-fg-muted">
      <Code2 size={28} className="text-fg-subtle" aria-hidden="true" />
      <p>{hasSnippets ? strings.snippets.noResults : strings.snippets.empty}</p>
    </div>
  );
}
