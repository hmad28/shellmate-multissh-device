import { Minus, Square, X } from 'lucide-react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { strings } from '@/i18n/en';
import { cn } from '@/lib/cn';

export function TitleBar() {
  const handleMinimize = () => {
    void getCurrentWindow().minimize();
  };
  const handleMaximize = () => {
    void getCurrentWindow().toggleMaximize();
  };
  const handleClose = () => {
    void getCurrentWindow().close();
  };

  return (
    <header
      className={cn(
        'titlebar-drag flex h-9 select-none items-center justify-between',
        'border-b border-border bg-bg-sidebar px-3 text-xs text-fg-muted',
      )}
    >
      <div className="flex items-center gap-2">
        <ShellMateMark />
        <span className="font-medium text-fg">{strings.app.name}</span>
      </div>

      <div className="titlebar-no-drag flex h-full items-stretch">
        <TitleBarButton
          label={strings.titlebar.minimize}
          onClick={handleMinimize}
        >
          <Minus size={14} />
        </TitleBarButton>
        <TitleBarButton
          label={strings.titlebar.maximize}
          onClick={handleMaximize}
        >
          <Square size={12} />
        </TitleBarButton>
        <TitleBarButton
          label={strings.titlebar.close}
          onClick={handleClose}
          variant="danger"
        >
          <X size={14} />
        </TitleBarButton>
      </div>
    </header>
  );
}

function TitleBarButton({
  children,
  label,
  onClick,
  variant = 'default',
}: {
  children: React.ReactNode;
  label: string;
  onClick: () => void;
  variant?: 'default' | 'danger';
}) {
  return (
    <button
      type="button"
      aria-label={label}
      onClick={onClick}
      className={cn(
        'flex w-10 items-center justify-center transition-colors',
        variant === 'default' && 'hover:bg-bg-elevated',
        variant === 'danger' && 'hover:bg-status-disconnected hover:text-white',
      )}
    >
      {children}
    </button>
  );
}

function ShellMateMark() {
  return (
    <svg
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="text-accent"
      aria-hidden="true"
    >
      <polyline points="4 17 10 11 4 5" />
      <line x1="12" y1="19" x2="20" y2="19" />
    </svg>
  );
}
