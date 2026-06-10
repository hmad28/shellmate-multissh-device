import { forwardRef } from 'react';
import { cn } from '@/lib/cn';

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  invalid?: boolean;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(function Input(
  { className, invalid, ...props },
  ref,
) {
  return (
    <input
      ref={ref}
      aria-invalid={invalid || undefined}
      className={cn(
        'w-full rounded-md border bg-bg-elevated px-3 py-2 text-sm text-fg',
        'placeholder:text-fg-subtle',
        'focus:outline-none focus:ring-1',
        invalid
          ? 'border-status-disconnected focus:border-status-disconnected focus:ring-status-disconnected'
          : 'border-border-subtle focus:border-accent focus:ring-accent',
        className,
      )}
      {...props}
    />
  );
});

interface TextareaProps extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  invalid?: boolean;
}

export const Textarea = forwardRef<HTMLTextAreaElement, TextareaProps>(
  function Textarea({ className, invalid, ...props }, ref) {
    return (
      <textarea
        ref={ref}
        aria-invalid={invalid || undefined}
        className={cn(
          'w-full resize-y rounded-md border bg-bg-elevated px-3 py-2 text-sm text-fg',
          'placeholder:text-fg-subtle',
          'focus:outline-none focus:ring-1',
          invalid
            ? 'border-status-disconnected focus:border-status-disconnected focus:ring-status-disconnected'
            : 'border-border-subtle focus:border-accent focus:ring-accent',
          className,
        )}
        {...props}
      />
    );
  },
);

interface SelectProps extends React.SelectHTMLAttributes<HTMLSelectElement> {
  invalid?: boolean;
}

export const Select = forwardRef<HTMLSelectElement, SelectProps>(
  function Select({ className, invalid, children, ...props }, ref) {
    return (
      <select
        ref={ref}
        aria-invalid={invalid || undefined}
        className={cn(
          'w-full rounded-md border bg-bg-elevated px-3 py-2 text-sm text-fg',
          'focus:outline-none focus:ring-1',
          invalid
            ? 'border-status-disconnected focus:border-status-disconnected focus:ring-status-disconnected'
            : 'border-border-subtle focus:border-accent focus:ring-accent',
          className,
        )}
        {...props}
      >
        {children}
      </select>
    );
  },
);

interface FieldProps {
  label: string;
  htmlFor: string;
  error?: string | null;
  hint?: string | undefined;
  children: React.ReactNode;
}

export function Field({ label, htmlFor, error, hint, children }: FieldProps) {
  return (
    <div>
      <label htmlFor={htmlFor} className="mb-1 block text-xs text-fg-muted">
        {label}
      </label>
      {children}
      {error && (
        <p role="alert" className="mt-1 text-xs text-status-disconnected">
          {error}
        </p>
      )}
      {!error && hint && <p className="mt-1 text-xs text-fg-subtle">{hint}</p>}
    </div>
  );
}
