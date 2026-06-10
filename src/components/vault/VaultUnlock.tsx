import { useState } from 'react';
import { strings } from '@/i18n/en';
import { cn } from '@/lib/cn';
import { useVaultStore } from '@/stores/vault-store';

export function VaultUnlock() {
  const { unlock, loading, error } = useVaultStore();
  const [password, setPassword] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (password.length === 0 || loading) return;
    try {
      await unlock(password);
      setPassword('');
    } catch {
      // surface from store
    }
  };

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-4" noValidate>
      <div>
        <label htmlFor="vault-unlock-password" className="sr-only">
          {strings.vault.masterPasswordPlaceholder}
        </label>
        <input
          id="vault-unlock-password"
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          placeholder={strings.vault.masterPasswordPlaceholder}
          autoFocus
          autoComplete="current-password"
          className={cn(
            'w-full rounded-md border border-border-subtle bg-bg-elevated px-3 py-2 text-sm',
            'text-fg placeholder:text-fg-subtle',
            'focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent',
          )}
        />
      </div>

      {error && (
        <p
          role="alert"
          className="border-status-disconnected/40 bg-status-disconnected/10 rounded-md border p-2 text-xs text-status-disconnected"
        >
          {error}
        </p>
      )}

      <button
        type="submit"
        disabled={password.length === 0 || loading}
        className={cn(
          'rounded-md bg-accent px-4 py-2 text-sm font-medium text-white',
          'hover:bg-accent-hover disabled:cursor-not-allowed disabled:opacity-50',
          'transition-colors',
        )}
      >
        {loading ? strings.vault.unlocking : strings.vault.unlockButton}
      </button>
    </form>
  );
}
