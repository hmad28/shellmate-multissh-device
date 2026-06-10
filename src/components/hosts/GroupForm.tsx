import { useEffect, useState } from 'react';
import { strings } from '@/i18n/en';
import { Button } from '@/components/ui/Button';
import { Field, Input } from '@/components/ui/Form';
import { Modal } from '@/components/ui/Modal';
import { useHostStore } from '@/stores/host-store';
import type { Group, GroupInput } from '@/types';

interface GroupFormProps {
  open: boolean;
  onClose: () => void;
  group?: Group | null;
}

const PRESET_COLORS = [
  '#ef4444', // red
  '#f59e0b', // amber
  '#22c55e', // green
  '#3b82f6', // blue
  '#a855f7', // purple
  '#ec4899', // pink
];

export function GroupForm({ open, onClose, group }: GroupFormProps) {
  const addGroup = useHostStore((s) => s.addGroup);
  const updateGroup = useHostStore((s) => s.updateGroup);

  const isEdit = !!group;
  const [name, setName] = useState('');
  const [color, setColor] = useState<string>('');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    setName(group?.name ?? '');
    setColor(group?.color ?? '');
    setError(null);
  }, [open, group]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim() || submitting) return;
    setSubmitting(true);
    setError(null);
    try {
      const input: GroupInput = {
        name: name.trim(),
        color: color || null,
        parentId: group?.parentId ?? null,
        sortOrder: group?.sortOrder ?? null,
      };
      if (isEdit) {
        await updateGroup(group!.id, input);
      } else {
        await addGroup(input);
      }
      onClose();
    } catch (err) {
      setError(String(err));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Modal
      open={open}
      onClose={onClose}
      title={isEdit ? strings.groupForm.editTitle : strings.groupForm.addTitle}
      size="sm"
    >
      <form onSubmit={handleSubmit} className="flex flex-col gap-3">
        <Field label={strings.groupForm.name} htmlFor="group-name">
          <Input
            id="group-name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder={strings.groupForm.namePlaceholder}
            autoFocus
            required
          />
        </Field>

        <Field
          label={strings.groupForm.color}
          htmlFor="group-color"
          hint={strings.groupForm.colorOptional}
        >
          <div className="flex flex-wrap items-center gap-2">
            <button
              type="button"
              onClick={() => setColor('')}
              aria-label="No color"
              className={`flex h-7 w-7 items-center justify-center rounded-full border ${
                color === ''
                  ? 'border-accent ring-2 ring-accent/40'
                  : 'border-border-strong'
              } bg-bg-elevated text-xs text-fg-subtle`}
            >
              ×
            </button>
            {PRESET_COLORS.map((c) => (
              <button
                key={c}
                type="button"
                onClick={() => setColor(c)}
                aria-label={`Color ${c}`}
                style={{ backgroundColor: c }}
                className={`h-7 w-7 rounded-full border-2 ${
                  color === c ? 'border-fg' : 'border-transparent'
                }`}
              />
            ))}
            <Input
              id="group-color"
              type="text"
              value={color}
              onChange={(e) => setColor(e.target.value)}
              placeholder="#3b82f6"
              className="ml-1 max-w-[110px] font-mono text-xs"
            />
          </div>
        </Field>

        {error && (
          <p
            role="alert"
            className="rounded-md border border-status-disconnected/40 bg-status-disconnected/10 p-2 text-xs text-status-disconnected"
          >
            {error}
          </p>
        )}

        <div className="mt-1 flex justify-end gap-2">
          <Button type="button" variant="secondary" onClick={onClose}>
            {strings.groupForm.cancel}
          </Button>
          <Button
            type="submit"
            variant="primary"
            disabled={submitting || !name.trim()}
          >
            {strings.groupForm.save}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
