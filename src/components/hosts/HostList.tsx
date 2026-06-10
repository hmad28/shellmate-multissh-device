import { useMemo, useState } from 'react';
import { ChevronRight, Plus, Server } from 'lucide-react';
import { strings } from '@/i18n/en';
import { cn } from '@/lib/cn';
import { useHostStore } from '@/stores/host-store';
import type { Group, Host } from '@/types';
import { HostForm } from './HostForm';
import { GroupForm } from './GroupForm';
import { HostItem } from './HostItem';

interface GroupSection {
  group: Group | null; // null = ungrouped
  hosts: Host[];
}

export function HostList() {
  const hosts = useHostStore((s) => s.hosts);
  const groups = useHostStore((s) => s.groups);
  const searchQuery = useHostStore((s) => s.searchQuery);
  const expandedGroups = useHostStore((s) => s.expandedGroups);
  const toggleGroupExpanded = useHostStore((s) => s.toggleGroupExpanded);
  const moveHostToGroup = useHostStore((s) => s.moveHostToGroup);

  const [hostFormOpen, setHostFormOpen] = useState(false);
  const [editingHost, setEditingHost] = useState<Host | null>(null);
  const [groupFormOpen, setGroupFormOpen] = useState(false);
  const [editingGroup, setEditingGroup] = useState<Group | null>(null);
  const [dragOverGroupId, setDragOverGroupId] = useState<
    string | null | 'ungrouped'
  >(null);

  // Compute filtered + grouped sections
  const sections = useMemo<GroupSection[]>(() => {
    const q = searchQuery.trim().toLowerCase();
    const filteredHosts = q
      ? hosts.filter((h) => {
          const groupName = groups.find((g) => g.id === h.groupId)?.name ?? '';
          return [
            h.label,
            h.hostname,
            h.username,
            groupName,
            ...h.tags,
            h.notes ?? '',
          ]
            .join(' ')
            .toLowerCase()
            .includes(q);
        })
      : hosts;

    const byGroup = new Map<string | null, Host[]>();
    for (const h of filteredHosts) {
      const key = h.groupId;
      const list = byGroup.get(key) ?? [];
      list.push(h);
      byGroup.set(key, list);
    }
    for (const list of byGroup.values()) {
      list.sort((a, b) => a.label.localeCompare(b.label));
    }

    const result: GroupSection[] = groups
      .slice()
      .sort((a, b) => a.sortOrder - b.sortOrder || a.name.localeCompare(b.name))
      .map((g) => ({ group: g, hosts: byGroup.get(g.id) ?? [] }));

    const ungroupedHosts = byGroup.get(null) ?? [];
    if (ungroupedHosts.length > 0 || groups.length === 0) {
      result.push({ group: null, hosts: ungroupedHosts });
    }
    return result;
  }, [hosts, groups, searchQuery]);

  const isEmpty = hosts.length === 0;
  const hasNoMatches = !isEmpty && sections.every((s) => s.hosts.length === 0);

  const handleAddHost = () => {
    setEditingHost(null);
    setHostFormOpen(true);
  };

  const handleEditHost = (host: Host) => {
    setEditingHost(host);
    setHostFormOpen(true);
  };

  const handleAddGroup = () => {
    setEditingGroup(null);
    setGroupFormOpen(true);
  };

  const handleEditGroup = (group: Group) => {
    setEditingGroup(group);
    setGroupFormOpen(true);
  };

  // Drag-and-drop: drop host on group section
  const handleDragOver = (e: React.DragEvent, groupId: string | null) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    setDragOverGroupId(groupId === null ? 'ungrouped' : groupId);
  };

  const handleDragLeave = () => setDragOverGroupId(null);

  const handleDrop = async (e: React.DragEvent, groupId: string | null) => {
    e.preventDefault();
    const hostId = e.dataTransfer.getData('application/x-shellmate-host');
    setDragOverGroupId(null);
    if (!hostId) return;
    const host = hosts.find((h) => h.id === hostId);
    if (!host || host.groupId === groupId) return;
    try {
      await moveHostToGroup(hostId, groupId);
    } catch (err) {
      console.error('Failed to move host', err);
    }
  };

  return (
    <>
      <div className="flex flex-1 flex-col overflow-hidden">
        <nav className="flex-1 overflow-y-auto p-2" aria-label="Host groups">
          {isEmpty ? (
            <EmptyState />
          ) : hasNoMatches ? (
            <NoMatchesState />
          ) : (
            sections.map((section) => {
              const sectionId = section.group?.id ?? 'ungrouped';
              const isExpanded =
                section.group === null
                  ? true // always expanded for ungrouped
                  : !!searchQuery || expandedGroups.has(section.group.id);
              const isDragOver =
                dragOverGroupId === (section.group?.id ?? 'ungrouped');

              return (
                <div
                  key={sectionId}
                  className={cn(
                    'mb-1 rounded',
                    isDragOver && 'bg-accent/10 ring-1 ring-accent',
                  )}
                  onDragOver={(e) =>
                    handleDragOver(e, section.group?.id ?? null)
                  }
                  onDragLeave={handleDragLeave}
                  onDrop={(e) => handleDrop(e, section.group?.id ?? null)}
                >
                  <GroupHeader
                    group={section.group}
                    count={section.hosts.length}
                    expanded={isExpanded}
                    onToggle={() =>
                      section.group && toggleGroupExpanded(section.group.id)
                    }
                    onEdit={() =>
                      section.group && handleEditGroup(section.group)
                    }
                  />
                  {isExpanded && (
                    <ul className="mt-0.5">
                      {section.hosts.map((h) => (
                        <li key={h.id}>
                          <HostItem host={h} onEdit={() => handleEditHost(h)} />
                        </li>
                      ))}
                    </ul>
                  )}
                </div>
              );
            })
          )}
        </nav>

        <div className="flex gap-1 border-t border-border-subtle p-2">
          <button
            type="button"
            onClick={handleAddHost}
            className={cn(
              'flex flex-1 items-center justify-center gap-1.5 rounded-md px-2 py-1.5 text-xs',
              'text-fg transition-colors hover:bg-bg-elevated',
            )}
            aria-label={strings.sidebar.addHost}
          >
            <Plus size={12} />
            <span>{strings.sidebar.addHost}</span>
          </button>
          <button
            type="button"
            onClick={handleAddGroup}
            className={cn(
              'flex items-center justify-center gap-1.5 rounded-md px-2 py-1.5 text-xs',
              'text-fg-muted transition-colors hover:bg-bg-elevated hover:text-fg',
            )}
            aria-label={strings.sidebar.addGroup}
            title={strings.sidebar.addGroup}
          >
            <Plus size={12} />
          </button>
        </div>
      </div>

      <HostForm
        open={hostFormOpen}
        onClose={() => setHostFormOpen(false)}
        host={editingHost}
      />
      <GroupForm
        open={groupFormOpen}
        onClose={() => setGroupFormOpen(false)}
        group={editingGroup}
      />
    </>
  );
}

function GroupHeader({
  group,
  count,
  expanded,
  onToggle,
  onEdit,
}: {
  group: Group | null;
  count: number;
  expanded: boolean;
  onToggle: () => void;
  onEdit: () => void;
}) {
  if (group === null) {
    return (
      <div className="px-2 py-1 text-xs font-medium uppercase tracking-wider text-fg-subtle">
        {strings.sidebar.ungrouped}
        <span className="ml-1.5 text-fg-subtle">{count}</span>
      </div>
    );
  }

  return (
    <div className="group/header flex items-center gap-1 px-1 py-1">
      <button
        type="button"
        onClick={onToggle}
        aria-expanded={expanded}
        className={cn(
          'flex flex-1 items-center gap-1.5 rounded px-1 py-0.5 text-left',
          'text-xs font-medium uppercase tracking-wider text-fg-muted',
          'hover:text-fg',
        )}
      >
        <ChevronRight
          size={12}
          className={cn(
            'shrink-0 transition-transform',
            expanded && 'rotate-90',
          )}
        />
        {group.color && (
          <span
            className="inline-block h-2 w-2 shrink-0 rounded-full"
            style={{ backgroundColor: group.color }}
            aria-hidden="true"
          />
        )}
        <span className="truncate">{group.name}</span>
        <span className="ml-auto text-fg-subtle">{count}</span>
      </button>
      <button
        type="button"
        onClick={onEdit}
        aria-label={`Edit ${group.name}`}
        className={cn(
          'invisible flex h-5 w-5 items-center justify-center rounded text-fg-subtle',
          'hover:bg-bg-elevated hover:text-fg group-hover/header:visible',
        )}
      >
        <PencilDot />
      </button>
    </div>
  );
}

function PencilDot() {
  return (
    <svg
      width="10"
      height="10"
      viewBox="0 0 24 24"
      fill="currentColor"
      aria-hidden="true"
    >
      <circle cx="5" cy="12" r="2" />
      <circle cx="12" cy="12" r="2" />
      <circle cx="19" cy="12" r="2" />
    </svg>
  );
}

function EmptyState() {
  return (
    <div className="flex flex-col items-center gap-2 p-6 text-center text-xs text-fg-subtle">
      <Server size={24} className="text-fg-subtle" aria-hidden="true" />
      <p>{strings.sidebar.noHosts}</p>
    </div>
  );
}

function NoMatchesState() {
  return (
    <div className="p-6 text-center text-xs text-fg-subtle">
      {strings.sidebar.noResults}
    </div>
  );
}
