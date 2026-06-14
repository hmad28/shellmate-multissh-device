import { useState, useRef } from 'react';
import { tauri } from '@/lib/tauri';
import { useHostStore } from '@/stores/host-store';
import { useToastStore } from '@/stores/toast-store';

interface TermiusHost {
  id: string;
  label: string;
  hostname: string;
  port: number;
  username: string;
  authentication_type?: string;
  password?: string;
  identity_file?: string;
  passphrase?: string;
  group?: string;
  tags?: string[];
}

export function ImportPanel() {
  const [importing, setImporting] = useState(false);
  const [result, setResult] = useState<string>('');
  const fileRef = useRef<HTMLInputElement>(null);
  const addToast = useToastStore((s) => s.addToast);
  const { loadHosts } = useHostStore();

  const handleImport = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    setImporting(true);
    setResult('');

    try {
      const text = await file.text();
      const data = JSON.parse(text);

      // Termius export format detection.
      let hosts: TermiusHost[] = [];

      if (Array.isArray(data)) {
        // Direct array of hosts.
        hosts = data;
      } else if (data.hosts) {
        // { hosts: [...] } format.
        hosts = data.hosts;
      } else if (data.entities) {
        // Termius v2 format with entities.
        hosts = data.entities
          .filter((e: Record<string, unknown>) => e.entity_type === 'host' || e.hostname)
          .map((e: Record<string, unknown>) => ({
            id: e.id || '',
            label: e.label || e.hostname || '',
            hostname: e.hostname || '',
            port: e.port || 22,
            username: e.username || 'root',
            authentication_type: e.authentication_type || e.authType || 'password',
            password: e.password || (e.credential as Record<string, unknown>)?.password as string || undefined,
            identity_file: e.identity_file || (e.credential as Record<string, unknown>)?.privateKey as string || undefined,
            group: e.group || e.group_name,
            tags: e.tags || [],
          }));
      }

      if (hosts.length === 0) {
        setResult('No hosts found in file. Expected Termius JSON export format.');
        return;
      }

      let imported = 0;
      let errors = 0;

      for (const host of hosts) {
        try {
          // Create group if needed.
          let groupId: string | null = null;
          if (host.group) {
            try {
              const groups = await tauri.groups.list();
              const existing = groups.find((g) => g.name === host.group);
              if (existing) {
                groupId = existing.id;
              } else {
                const group = await tauri.groups.create({ name: host.group, color: '#6b7280', parentId: null, sortOrder: 0 });
                groupId = group.id;
              }
            } catch {
              // Ignore group creation errors.
            }
          }

          // Save credential.
          let credentialId = '';
          if (host.password) {
            credentialId = await tauri.credentials.save('password', host.password);
          } else if (host.identity_file) {
            credentialId = await tauri.credentials.save('private_key', host.identity_file);
          }

          if (!credentialId) {
            errors++;
            continue;
          }

          // Create host.
          await tauri.hosts.create({
            label: host.label || host.hostname,
            hostname: host.hostname,
            port: host.port || 22,
            username: host.username || 'root',
            authType: host.identity_file ? 'key' : 'password',
            credentialId,
            groupId,
            tags: host.tags || [],
            notes: '',
          });

          imported++;
        } catch {
          errors++;
        }
      }

      await loadHosts();
      setResult(`Imported ${imported} hosts. ${errors > 0 ? `${errors} errors.` : ''}`);
      addToast('success', `Imported ${imported} hosts from Termius`);
    } catch (err) {
      setResult(`Error: ${String(err)}`);
      addToast('error', 'Failed to import hosts');
    } finally {
      setImporting(false);
      if (fileRef.current) fileRef.current.value = '';
    }
  };

  return (
    <div className="p-4">
      <h2 className="mb-4 text-lg font-semibold text-[var(--color-fg)]">Import Hosts</h2>
      <p className="mb-4 text-sm text-[var(--color-fg-muted)]">
        Import hosts from Termius, MobaXterm, or other SSH clients. Supports JSON export format.
      </p>

      <div className="mb-4">
        <input
          ref={fileRef}
          type="file"
          accept=".json"
          onChange={handleImport}
          disabled={importing}
          className="block w-full text-sm text-[var(--color-fg-muted)]
            file:mr-4 file:rounded file:border-0 file:bg-[var(--color-accent)] file:px-3 file:py-1.5 file:text-sm file:text-white
            hover:file:bg-[var(--color-accent-hover)]"
        />
      </div>

      {importing && (
        <div className="text-sm text-[var(--color-fg-muted)]">Importing...</div>
      )}

      {result && (
        <div className={`rounded p-2 text-sm ${result.startsWith('Error') ? 'bg-[var(--color-status-disconnected)]/10 text-[var(--color-status-disconnected)]' : 'bg-[var(--color-status-connected)]/10 text-[var(--color-status-connected)]'}`}>
          {result}
        </div>
      )}
    </div>
  );
}
