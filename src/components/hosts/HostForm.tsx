import { useEffect, useState } from 'react';
import { strings } from '@/i18n/en';
import { Button } from '@/components/ui/Button';
import { Field, Input, Select, Textarea } from '@/components/ui/Form';
import { Modal } from '@/components/ui/Modal';
import { tauri } from '@/lib/tauri';
import { useHostStore } from '@/stores/host-store';
import type { AuthType, Host, HostInput } from '@/types';

interface HostFormProps {
  open: boolean;
  onClose: () => void;
  /** When set, form is in edit mode. */
  host?: Host | null;
  /** When set, populates the form but leaves it in create mode. */
  initialData?: Partial<FormState> | undefined;
  /** Called after successful save. */
  onSaved?: (host: Host) => void;
}

interface FormState {
  label: string;
  hostname: string;
  port: string;
  username: string;
  authType: AuthType;
  /** plaintext credential entered by user; never persisted in plaintext */
  credentialPlaintext: string;
  groupId: string;
  tags: string;
  notes: string;
}

const emptyForm: FormState = {
  label: '',
  hostname: '',
  port: '22',
  username: '',
  authType: 'password',
  credentialPlaintext: '',
  groupId: '',
  tags: '',
  notes: '',
};

export function HostForm({
  open,
  onClose,
  host,
  initialData,
  onSaved,
}: HostFormProps) {
  const groups = useHostStore((s) => s.groups);
  const addHost = useHostStore((s) => s.addHost);
  const updateHost = useHostStore((s) => s.updateHost);

  const isEdit = host !== null && host !== undefined;
  const [form, setForm] = useState<FormState>(emptyForm);
  const [errors, setErrors] = useState<
    Partial<Record<keyof FormState, string>>
  >({});
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (!open) return;
    if (host) {
      setForm({
        label: host.label,
        hostname: host.hostname,
        port: String(host.port),
        username: host.username,
        authType: host.authType,
        credentialPlaintext: '', // never reload — user must re-enter to change
        groupId: host.groupId ?? '',
        tags: host.tags.join(', '),
        notes: host.notes ?? '',
      });
    } else if (initialData) {
      setForm({ ...emptyForm, ...initialData });
    } else {
      setForm(emptyForm);
    }
    setErrors({});
    setSubmitError(null);
  }, [open, host, initialData]);

  const update = <K extends keyof FormState>(key: K, value: FormState[K]) => {
    setForm((s) => ({ ...s, [key]: value }));
    if (errors[key]) setErrors((e) => ({ ...e, [key]: undefined }));
  };

  const validate = (): boolean => {
    const next: typeof errors = {};
    if (!form.label.trim()) next.label = strings.hostForm.requiredLabel;
    if (!form.hostname.trim())
      next.hostname = strings.hostForm.requiredHostname;
    if (!form.username.trim())
      next.username = strings.hostForm.requiredUsername;

    const portNum = Number(form.port);
    if (!Number.isInteger(portNum) || portNum < 1 || portNum > 65535) {
      next.port = strings.hostForm.requiredPort;
    }

    // For new hosts, credential is required. For edits, blank means "keep existing".
    if (!isEdit && form.credentialPlaintext.trim().length === 0) {
      next.credentialPlaintext = strings.hostForm.requiredCredential;
    }

    setErrors(next);
    return Object.keys(next).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!validate() || submitting) return;
    setSubmitting(true);
    setSubmitError(null);

    try {
      const tags = form.tags
        .split(',')
        .map((t) => t.trim())
        .filter(Boolean);

      // Resolve credential id:
      //  - new host: save credential, get id
      //  - edit + new credential entered: save new, replace
      //  - edit + no new credential: keep existing id
      let credentialId: string;
      const credType: 'password' | 'private_key' =
        form.authType === 'password' ? 'password' : 'private_key';

      if (isEdit && form.credentialPlaintext.trim().length === 0) {
        credentialId = host!.credentialId;
      } else {
        credentialId = await tauri.credentials.save(
          credType,
          form.credentialPlaintext,
        );
      }

      const input: HostInput = {
        label: form.label.trim(),
        hostname: form.hostname.trim(),
        port: Number(form.port),
        username: form.username.trim(),
        authType: form.authType,
        credentialId,
        groupId: form.groupId === '' ? null : form.groupId,
        tags,
        notes: form.notes.trim() === '' ? null : form.notes.trim(),
      };

      const saved = isEdit
        ? await updateHost(host!.id, input)
        : await addHost(input);

      onSaved?.(saved);
      onClose();
    } catch (err) {
      setSubmitError(String(err));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Modal
      open={open}
      onClose={onClose}
      title={isEdit ? strings.hostForm.editTitle : strings.hostForm.addTitle}
      size="md"
    >
      <form onSubmit={handleSubmit} className="flex flex-col gap-3" noValidate>
        <Field
          label={strings.hostForm.label}
          htmlFor="host-label"
          error={errors.label ?? null}
        >
          <Input
            id="host-label"
            value={form.label}
            onChange={(e) => update('label', e.target.value)}
            placeholder={strings.hostForm.labelPlaceholder}
            invalid={!!errors.label}
            autoFocus
          />
        </Field>

        <div className="grid grid-cols-[1fr_120px] gap-3">
          <Field
            label={strings.hostForm.hostname}
            htmlFor="host-hostname"
            error={errors.hostname ?? null}
          >
            <Input
              id="host-hostname"
              value={form.hostname}
              onChange={(e) => update('hostname', e.target.value)}
              placeholder={strings.hostForm.hostnamePlaceholder}
              invalid={!!errors.hostname}
              autoComplete="off"
              spellCheck={false}
            />
          </Field>
          <Field
            label={strings.hostForm.port}
            htmlFor="host-port"
            error={errors.port ?? null}
          >
            <Input
              id="host-port"
              type="number"
              min={1}
              max={65535}
              value={form.port}
              onChange={(e) => update('port', e.target.value)}
              invalid={!!errors.port}
            />
          </Field>
        </div>

        <Field
          label={strings.hostForm.username}
          htmlFor="host-username"
          error={errors.username ?? null}
        >
          <Input
            id="host-username"
            value={form.username}
            onChange={(e) => update('username', e.target.value)}
            invalid={!!errors.username}
            autoComplete="off"
            spellCheck={false}
          />
        </Field>

        <Field label={strings.hostForm.auth} htmlFor="host-auth">
          <Select
            id="host-auth"
            value={form.authType}
            onChange={(e) => update('authType', e.target.value as AuthType)}
          >
            <option value="password">{strings.hostForm.authPassword}</option>
            <option value="key">{strings.hostForm.authKey}</option>
            <option value="key_passphrase">
              {strings.hostForm.authKeyPassphrase}
            </option>
          </Select>
        </Field>

        {form.authType === 'password' ? (
          <Field
            label={strings.hostForm.password}
            htmlFor="host-password"
            error={errors.credentialPlaintext ?? null}
            hint={isEdit ? 'Leave blank to keep existing' : undefined}
          >
            <Input
              id="host-password"
              type="password"
              value={form.credentialPlaintext}
              onChange={(e) => update('credentialPlaintext', e.target.value)}
              invalid={!!errors.credentialPlaintext}
              autoComplete="new-password"
            />
          </Field>
        ) : (
          <Field
            label={strings.hostForm.privateKey}
            htmlFor="host-privkey"
            error={errors.credentialPlaintext ?? null}
            hint={isEdit ? 'Leave blank to keep existing' : undefined}
          >
            <Textarea
              id="host-privkey"
              rows={5}
              value={form.credentialPlaintext}
              onChange={(e) => update('credentialPlaintext', e.target.value)}
              placeholder={strings.hostForm.privateKeyPlaceholder}
              invalid={!!errors.credentialPlaintext}
              className="font-mono text-xs"
            />
          </Field>
        )}

        <Field label={strings.hostForm.group} htmlFor="host-group">
          <Select
            id="host-group"
            value={form.groupId}
            onChange={(e) => update('groupId', e.target.value)}
          >
            <option value="">{strings.hostForm.groupNone}</option>
            {groups.map((g) => (
              <option key={g.id} value={g.id}>
                {g.name}
              </option>
            ))}
          </Select>
        </Field>

        <Field label={strings.hostForm.tags} htmlFor="host-tags">
          <Input
            id="host-tags"
            value={form.tags}
            onChange={(e) => update('tags', e.target.value)}
            placeholder={strings.hostForm.tagsPlaceholder}
          />
        </Field>

        <Field label={strings.hostForm.notes} htmlFor="host-notes">
          <Textarea
            id="host-notes"
            rows={2}
            value={form.notes}
            onChange={(e) => update('notes', e.target.value)}
            placeholder={strings.hostForm.notesPlaceholder}
          />
        </Field>

        {submitError && (
          <p
            role="alert"
            className="border-status-disconnected/40 bg-status-disconnected/10 rounded-md border p-2 text-xs text-status-disconnected"
          >
            {submitError}
          </p>
        )}

        <div className="mt-1 flex justify-end gap-2">
          <Button type="button" variant="secondary" onClick={onClose}>
            {strings.hostForm.cancel}
          </Button>
          <Button type="submit" variant="primary" disabled={submitting}>
            {submitting ? strings.hostForm.saving : strings.hostForm.save}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
