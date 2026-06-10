import { useState } from 'react';
import { strings } from '@/i18n/en';
import { Button } from '@/components/ui/Button';
import { Field, Input, Select } from '@/components/ui/Form';
import { tauri } from '@/lib/tauri';
import { useSettingsStore } from '@/stores/settings-store';
import { useVaultStore } from '@/stores/vault-store';

const AUTOLOCK_OPTIONS = [
  { value: 0, label: 'Never' },
  { value: 60, label: '1 minute' },
  { value: 5 * 60, label: '5 minutes' },
  { value: 15 * 60, label: '15 minutes' },
  { value: 30 * 60, label: '30 minutes' },
  { value: 60 * 60, label: '1 hour' },
];

export function VaultSettingsTab() {
  const settings = useSettingsStore((s) => s.settings);
  const setAutolockSecs = useSettingsStore((s) => s.setAutolockSecs);
  const lock = useVaultStore((s) => s.lock);

  const [currentPw, setCurrentPw] = useState('');
  const [newPw, setNewPw] = useState('');
  const [confirmPw, setConfirmPw] = useState('');
  const [pwSubmitting, setPwSubmitting] = useState(false);
  const [pwSuccess, setPwSuccess] = useState<string | null>(null);
  const [pwError, setPwError] = useState<string | null>(null);

  const tooShort = newPw.length > 0 && newPw.length < 12;
  const mismatch = confirmPw.length > 0 && newPw !== confirmPw;
  const canChange =
    !pwSubmitting &&
    currentPw.length > 0 &&
    newPw.length >= 12 &&
    newPw === confirmPw;

  const handleChange = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!canChange) return;
    setPwSubmitting(true);
    setPwSuccess(null);
    setPwError(null);
    try {
      await tauri.vault.changeMasterPassword(currentPw, newPw);
      setPwSuccess(strings.settings.passwordChanged);
      setCurrentPw('');
      setNewPw('');
      setConfirmPw('');
    } catch (err) {
      setPwError(String(err));
    } finally {
      setPwSubmitting(false);
    }
  };

  return (
    <section className="flex flex-col gap-6">
      <div>
        <header className="mb-2">
          <h2 className="text-sm font-semibold text-fg">
            {strings.settings.vaultHeading}
          </h2>
          <p className="text-xs text-fg-muted">
            {strings.settings.vaultDescription}
          </p>
        </header>

        <Field label={strings.settings.autolock} htmlFor="setting-autolock">
          <Select
            id="setting-autolock"
            value={String(settings.autolockSecs)}
            onChange={(e) => void setAutolockSecs(Number(e.target.value))}
          >
            {AUTOLOCK_OPTIONS.map((o) => (
              <option key={o.value} value={o.value}>
                {o.label}
              </option>
            ))}
          </Select>
        </Field>

        <div className="mt-3">
          <Button variant="secondary" size="sm" onClick={() => void lock()}>
            {strings.settings.lockNow}
          </Button>
        </div>
      </div>

      <div>
        <header className="mb-2">
          <h3 className="text-sm font-semibold text-fg">
            {strings.settings.changePasswordHeading}
          </h3>
          <p className="text-xs text-fg-muted">
            {strings.settings.changePasswordDescription}
          </p>
        </header>

        <form
          onSubmit={handleChange}
          className="flex flex-col gap-3"
          noValidate
        >
          <Field
            label={strings.settings.currentPassword}
            htmlFor="setting-currentPw"
          >
            <Input
              id="setting-currentPw"
              type="password"
              value={currentPw}
              onChange={(e) => setCurrentPw(e.target.value)}
              autoComplete="current-password"
            />
          </Field>

          <Field
            label={strings.settings.newPassword}
            htmlFor="setting-newPw"
            error={tooShort ? strings.vault.tooShort : null}
            hint={strings.vault.minLengthHint}
          >
            <Input
              id="setting-newPw"
              type="password"
              value={newPw}
              onChange={(e) => setNewPw(e.target.value)}
              invalid={tooShort}
              autoComplete="new-password"
            />
          </Field>

          <Field
            label={strings.settings.confirmNewPassword}
            htmlFor="setting-confirmPw"
            error={mismatch ? strings.vault.mismatch : null}
          >
            <Input
              id="setting-confirmPw"
              type="password"
              value={confirmPw}
              onChange={(e) => setConfirmPw(e.target.value)}
              invalid={mismatch}
              autoComplete="new-password"
            />
          </Field>

          {pwSuccess && (
            <p
              role="status"
              className="border-status-connected/40 bg-status-connected/10 rounded-md border p-2 text-xs text-status-connected"
            >
              {pwSuccess}
            </p>
          )}
          {pwError && (
            <p
              role="alert"
              className="border-status-disconnected/40 bg-status-disconnected/10 rounded-md border p-2 text-xs text-status-disconnected"
            >
              {pwError}
            </p>
          )}

          <div>
            <Button
              variant="primary"
              size="sm"
              type="submit"
              disabled={!canChange}
            >
              {pwSubmitting
                ? strings.settings.changingPassword
                : strings.settings.changePassword}
            </Button>
          </div>
        </form>
      </div>
    </section>
  );
}
