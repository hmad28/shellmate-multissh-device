import { useState } from 'react';
import { strings } from '@/i18n/en';
import { cn } from '@/lib/cn';
import { useSshStore } from '@/stores/ssh-store';
import { useTabStore } from '@/stores/tab-store';

type AuthMode = 'password' | 'key';

interface QuickConnectProps {
  onConnected?: () => void;
}

/**
 * One-off SSH connection form for testing during MVP. Credentials are sent
 * to the backend but NOT persisted (no credential row, no host row).
 */
export function QuickConnect({ onConnected }: QuickConnectProps) {
  const addTab = useTabStore((s) => s.addTab);

  const [hostname, setHostname] = useState('');
  const [port, setPort] = useState('22');
  const [username, setUsername] = useState('');
  const [authMode, setAuthMode] = useState<AuthMode>('password');
  const [password, setPassword] = useState('');
  const [privateKey, setPrivateKey] = useState('');
  const [passphrase, setPassphrase] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const portNum = Number(port);
  const portValid = Number.isInteger(portNum) && portNum > 0 && portNum < 65536;

  const canSubmit =
    !submitting &&
    hostname.trim().length > 0 &&
    username.trim().length > 0 &&
    portValid &&
    (authMode === 'password'
      ? password.length > 0
      : privateKey.trim().length > 0);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!canSubmit) return;
    setSubmitting(true);
    setError(null);

    const label = `${username}@${hostname}`;
    const tabId = addTab({ label });

    try {
      await useSshStore.getState().connectQuick(tabId, {
        hostname: hostname.trim(),
        port: portNum,
        username: username.trim(),
        label,
        auth:
          authMode === 'password'
            ? { type: 'password', password }
            : {
                type: 'key',
                privateKey,
                passphrase: passphrase.length > 0 ? passphrase : null,
              },
      });
      // Clear sensitive fields immediately after submit succeeds
      setPassword('');
      setPrivateKey('');
      setPassphrase('');
      onConnected?.();
    } catch (err) {
      setError(String(err));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <form
      onSubmit={handleSubmit}
      className="flex flex-col gap-3 p-4"
      aria-label={strings.quickConnect.title}
    >
      <header className="mb-1">
        <h2 className="text-sm font-semibold text-fg">
          {strings.quickConnect.title}
        </h2>
        <p className="text-xs text-fg-muted">{strings.quickConnect.subtitle}</p>
      </header>

      <div className="grid grid-cols-[1fr_auto] gap-2">
        <Field
          id="qc-hostname"
          label={strings.quickConnect.hostnameLabel}
          value={hostname}
          onChange={setHostname}
          autoFocus
        />
        <Field
          id="qc-port"
          label={strings.quickConnect.portLabel}
          value={port}
          onChange={setPort}
          width="w-20"
          type="number"
        />
      </div>

      <Field
        id="qc-username"
        label={strings.quickConnect.usernameLabel}
        value={username}
        onChange={setUsername}
      />

      <fieldset className="flex flex-col gap-2">
        <legend className="text-xs text-fg-muted">
          {strings.quickConnect.authLabel}
        </legend>
        <div className="flex gap-3 text-xs">
          <RadioOption
            label={strings.quickConnect.authPassword}
            checked={authMode === 'password'}
            onChange={() => setAuthMode('password')}
          />
          <RadioOption
            label={strings.quickConnect.authKey}
            checked={authMode === 'key'}
            onChange={() => setAuthMode('key')}
          />
        </div>
      </fieldset>

      {authMode === 'password' ? (
        <Field
          id="qc-password"
          label={strings.quickConnect.passwordLabel}
          value={password}
          onChange={setPassword}
          type="password"
        />
      ) : (
        <>
          <div>
            <label
              htmlFor="qc-private-key"
              className="mb-1 block text-xs text-fg-muted"
            >
              {strings.quickConnect.privateKeyLabel}
            </label>
            <textarea
              id="qc-private-key"
              value={privateKey}
              onChange={(e) => setPrivateKey(e.target.value)}
              placeholder={strings.quickConnect.privateKeyPlaceholder}
              rows={5}
              className={cn(
                'w-full resize-y rounded-md border border-border-subtle bg-bg-elevated px-3 py-2 font-mono text-xs',
                'text-fg placeholder:text-fg-subtle',
                'focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent',
              )}
            />
          </div>
          <Field
            id="qc-passphrase"
            label={strings.quickConnect.passphraseLabel}
            value={passphrase}
            onChange={setPassphrase}
            type="password"
          />
        </>
      )}

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
        disabled={!canSubmit}
        className={cn(
          'mt-1 rounded-md bg-accent px-4 py-2 text-sm font-medium text-white',
          'hover:bg-accent-hover disabled:cursor-not-allowed disabled:opacity-50',
          'transition-colors',
        )}
      >
        {submitting
          ? strings.quickConnect.connecting
          : strings.quickConnect.connect}
      </button>
    </form>
  );
}

function Field({
  id,
  label,
  value,
  onChange,
  type = 'text',
  width,
  autoFocus,
}: {
  id: string;
  label: string;
  value: string;
  onChange: (v: string) => void;
  type?: string;
  width?: string;
  autoFocus?: boolean;
}) {
  return (
    <div className={width}>
      <label htmlFor={id} className="mb-1 block text-xs text-fg-muted">
        {label}
      </label>
      <input
        id={id}
        type={type}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        autoFocus={autoFocus}
        autoComplete="off"
        className={cn(
          'w-full rounded-md border border-border-subtle bg-bg-elevated px-3 py-2 text-sm text-fg',
          'placeholder:text-fg-subtle',
          'focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent',
        )}
      />
    </div>
  );
}

function RadioOption({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: () => void;
}) {
  return (
    <label className="inline-flex items-center gap-1.5 text-fg-muted">
      <input
        type="radio"
        checked={checked}
        onChange={onChange}
        className="accent-accent"
      />
      <span className={checked ? 'text-fg' : ''}>{label}</span>
    </label>
  );
}
