import { useEffect, useState } from 'react';
import { strings } from '@/i18n/en';
import { Button } from '@/components/ui/Button';
import { Field, Input, Textarea } from '@/components/ui/Form';
import { Modal } from '@/components/ui/Modal';
import { useSnippetStore } from '@/stores/snippet-store';
import type { Snippet, SnippetInput } from '@/types/snippet';

interface SnippetFormProps {
  open: boolean;
  onClose: () => void;
  snippet?: Snippet | null;
}

export function SnippetForm({ open, onClose, snippet }: SnippetFormProps) {
  const add = useSnippetStore((s) => s.add);
  const update = useSnippetStore((s) => s.update);

  const isEdit = !!snippet;
  const [title, setTitle] = useState('');
  const [command, setCommand] = useState('');
  const [description, setDescription] = useState('');
  const [tags, setTags] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    setTitle(snippet?.title ?? '');
    setCommand(snippet?.command ?? '');
    setDescription(snippet?.description ?? '');
    setTags(snippet?.tags.join(', ') ?? '');
    setError(null);
  }, [open, snippet]);

  const canSubmit = !submitting && title.trim() && command.trim();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!canSubmit) return;
    setSubmitting(true);
    setError(null);
    try {
      const input: SnippetInput = {
        title: title.trim(),
        command,
        description: description.trim() === '' ? null : description.trim(),
        tags: tags
          .split(',')
          .map((t) => t.trim())
          .filter(Boolean),
      };
      if (isEdit) {
        await update(snippet!.id, input);
      } else {
        await add(input);
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
      title={isEdit ? strings.snippets.editTitle : strings.snippets.addTitle}
      description={strings.snippets.formHint}
      size="md"
    >
      <form onSubmit={handleSubmit} className="flex flex-col gap-3" noValidate>
        <Field label={strings.snippets.titleLabel} htmlFor="snippet-title">
          <Input
            id="snippet-title"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            autoFocus
            required
          />
        </Field>

        <Field
          label={strings.snippets.commandLabel}
          htmlFor="snippet-command"
          hint={strings.snippets.commandHint}
        >
          <Textarea
            id="snippet-command"
            value={command}
            onChange={(e) => setCommand(e.target.value)}
            rows={4}
            className="font-mono text-xs"
            required
          />
        </Field>

        <Field
          label={strings.snippets.descriptionLabel}
          htmlFor="snippet-description"
        >
          <Textarea
            id="snippet-description"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            rows={2}
          />
        </Field>

        <Field label={strings.snippets.tagsLabel} htmlFor="snippet-tags">
          <Input
            id="snippet-tags"
            value={tags}
            onChange={(e) => setTags(e.target.value)}
            placeholder={strings.snippets.tagsPlaceholder}
          />
        </Field>

        {error && (
          <p
            role="alert"
            className="border-status-disconnected/40 bg-status-disconnected/10 rounded-md border p-2 text-xs text-status-disconnected"
          >
            {error}
          </p>
        )}

        <div className="mt-1 flex justify-end gap-2">
          <Button variant="secondary" onClick={onClose}>
            {strings.snippets.cancel}
          </Button>
          <Button variant="primary" type="submit" disabled={!canSubmit}>
            {submitting ? strings.snippets.saving : strings.snippets.save}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
