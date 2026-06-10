import { useState } from 'react';
import { AlertTriangle } from 'lucide-react';
import { strings } from '@/i18n/en';
import { cn } from '@/lib/cn';
import { useVaultStore } from '@/stores/vault-store';

export function VaultSetup() {
  const { setup, loading, error } = useVaultStore();
  const [password, setPassword] = useState('');
  const [confirm, setConfirm] = useState('');
  const [acknowledged, setAcknowledged] = useState(false);
  const [localError, setLocalError] = useState<string | null>(null);

  const tooShort = password.length > 0 && password.length < 12;
  const mismatch = confirm.length > 0 && password !== confirm;
  const canSubmit =
    !tooShort &&
    password.length >= 12 &&
    password === confirm &&
    acknowledged &&
    !loading;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLocalError(null);
    if (password.length < 12) {
      setLocalError(strings.vault.tooShort);
      return;
    }
    if (password !== confirm) {
      setLocalError(strings.vault.mismatch);
      return;
    }
    try {
      await setup(password);
    } catch {
      // surface from store
    }
  };

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-4" noValidate>
      <div
        className={cn(
          'border-status-connecting/40 bg-status-connecting/10 flex gap-3 rounded-md border p-3 text-xs',
        )}
        role="note"
      >
        <AlertTriangle
          size={16}
          className="mt-0.5 shrink-0 text-status-connecting"
          aria-hidden="true"
        />
        <div className="space-y-1 text-fg">
          <p className="font-medium">{strings.vault.setupWarningTitle}</p>
          <p className="text-fg-muted">{strings.vault.setupWarning}</p>
        </div>
      </div>

      <Field
        id="master-password"
        label={strings.vault.masterPasswordPlaceholder}
        value={password}
        onChange={setPassword}
        autoFocus
        hint={strings.vault.minLengthHint}
        error={tooShort ? strings.vault.tooShort : null}
      />
      <Field
        id="confirm-password"
        label={strings.vault.confirmPasswordPlaceholder}
        value={confirm}
        onChange={setConfirm}
        error={mismatch ? strings.vault.mismatch : null}
      />

      <label className="flex items-start gap-2 text-xs text-fg-muted">
        <input
          type="checkbox"
          checked={acknowledged}
          onChange={(e) => setAcknowledged(e.target.checked)}
          className="mt-0.5 accent-accent"
        />
        <span>{strings.vault.setupConfirm}</span>
      </label>

      {(localError || error) && (
        <p
          role="alert"
          className="border-status-disconnected/40 bg-status-disconnected/10 rounded-md border p-2 text-xs text-status-disconnected"
        >
          {localError ?? error}
        </p>
      )}

      <button
        type="submit"
        disabled={!canSubmit}
        className={cn(
          'rounded-md bg-accent px-4 py-2 text-sm font-medium text-white',
          'hover:bg-accent-hover disabled:cursor-not-allowed disabled:opacity-50',
          'transition-colors',
        )}
      >
        {loading ? strings.vault.creating : strings.vault.create}
      </button>
    </form>
  );
}

function Field({
  id,
  label,
  value,
  onChange,
  hint,
  error,
  autoFocus,
}: {
  id: string;
  label: string;
  value: string;
  onChange: (v: string) => void;
  hint?: string;
  error?: string | null;
  autoFocus?: boolean;
}) {
  return (
    <div>
      <label htmlFor={id} className="mb-1 block text-xs text-fg-muted">
        {label}
      </label>
      <input
        id={id}
        type="password"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        autoFocus={autoFocus}
        autoComplete="new-password"
        aria-invalid={!!error}
        aria-describedby={
          error ? `${id}-error` : hint ? `${id}-hint` : undefined
        }
        className={cn(
          'w-full rounded-md border bg-bg-elevated px-3 py-2 text-sm text-fg',
          'placeholder:text-fg-subtle focus:outline-none focus:ring-1',
          error
            ? 'border-status-disconnected focus:ring-status-disconnected'
            : 'border-border-subtle focus:border-accent focus:ring-accent',
        )}
      />
      {error && (
        <p id={`${id}-error`} className="mt-1 text-xs text-status-disconnected">
          {error}
        </p>
      )}
      {!error && hint && (
        <p id={`${id}-hint`} className="mt-1 text-xs text-fg-subtle">
          {hint}
        </p>
      )}
    </div>
  );
}
