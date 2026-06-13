import { useToastStore, type Toast, type ToastType } from '@/stores/toast-store';

const typeStyles: Record<ToastType, string> = {
  success: 'border-[var(--color-status-connected)] bg-[var(--color-status-connected)]/10 text-[var(--color-status-connected)]',
  error: 'border-[var(--color-status-disconnected)] bg-[var(--color-status-disconnected)]/10 text-[var(--color-status-disconnected)]',
  warning: 'border-[var(--color-status-connecting)] bg-[var(--color-status-connecting)]/10 text-[var(--color-status-connecting)]',
  info: 'border-[var(--color-accent)] bg-[var(--color-accent)]/10 text-[var(--color-accent)]',
};

const typeIcons: Record<ToastType, string> = {
  success: '✓',
  error: '✕',
  warning: '⚠',
  info: 'ℹ',
};

function ToastItem({ toast }: { toast: Toast }) {
  const removeToast = useToastStore((s) => s.removeToast);

  return (
    <div
      className={`flex items-center gap-2 rounded-md border px-3 py-2 text-sm shadow-lg backdrop-blur-sm transition-all ${typeStyles[toast.type]}`}
    >
      <span className="font-bold">{typeIcons[toast.type]}</span>
      <span className="flex-1">{toast.message}</span>
      <button
        onClick={() => removeToast(toast.id)}
        className="ml-2 opacity-60 hover:opacity-100"
      >
        ✕
      </button>
    </div>
  );
}

export function ToastContainer() {
  const toasts = useToastStore((s) => s.toasts);

  if (toasts.length === 0) return null;

  return (
    <div className="fixed right-4 top-4 z-[9999] flex w-80 flex-col gap-2">
      {toasts.map((toast) => (
        <ToastItem key={toast.id} toast={toast} />
      ))}
    </div>
  );
}
