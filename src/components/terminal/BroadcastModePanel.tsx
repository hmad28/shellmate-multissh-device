import { useState } from 'react';
import { Radio, X } from 'lucide-react';
import { strings } from '@/i18n/en';
import { cn } from '@/lib/cn';
import { Button } from '@/components/ui/Button';
import { useBroadcastStore } from '@/stores/broadcast-store';
import { useTabStore } from '@/stores/tab-store';
import { useSshStore } from '@/stores/ssh-store';

interface BroadcastModePanelProps {
  onClose: () => void;
}

export function BroadcastModePanel({ onClose }: BroadcastModePanelProps) {
  const { tabs } = useTabStore();
  const { sessionByTab } = useSshStore();
  const {
    broadcastSessions,
    addSession,
    removeSession,
    isSessionActive,
    sendToAll,
  } = useBroadcastStore();
  const [input, setInput] = useState('');

  const handleSend = async () => {
    if (!input.trim()) return;
    await sendToAll(input + '\n');
    setInput('');
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const toggleSession = async (sessionId: string) => {
    if (isSessionActive(sessionId)) {
      await removeSession(sessionId);
    } else {
      await addSession(sessionId);
    }
  };

  return (
    <div className="flex h-full flex-col bg-bg">
      <div className="flex items-center justify-between border-b border-border px-4 py-3">
        <div className="flex items-center gap-2">
          <Radio size={16} className="text-accent" />
          <h2 className="text-sm font-semibold text-fg">
            {strings.broadcast.title}
          </h2>
          <span className="text-xs text-fg-muted">
            ({broadcastSessions.size} {strings.broadcast.sessions})
          </span>
        </div>
        <button
          onClick={onClose}
          className="text-fg-muted transition-colors hover:text-fg"
        >
          <X size={16} />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-4">
        <div className="mb-4">
          <h3 className="mb-2 text-xs font-medium text-fg-muted">
            {strings.broadcast.selectSessions}
          </h3>
          <div className="space-y-2">
            {tabs.map((tab) => {
              const sessionId = sessionByTab[tab.id];
              if (!sessionId) return null;
              const isActive = isSessionActive(sessionId);
              return (
                <label
                  key={tab.id}
                  className={cn(
                    'flex cursor-pointer items-center gap-3 rounded-md p-2 transition-colors',
                    isActive
                      ? 'bg-accent/10 border-accent/30 border'
                      : 'border border-transparent bg-bg-sidebar hover:bg-bg-elevated',
                  )}
                >
                  <input
                    type="checkbox"
                    checked={isActive}
                    onChange={() => toggleSession(sessionId)}
                    className="rounded border-border"
                  />
                  <div className="min-w-0 flex-1">
                    <div className="truncate text-sm text-fg">{tab.label}</div>
                    <div className="truncate text-xs text-fg-muted">
                      {sessionId}
                    </div>
                  </div>
                  {isActive && (
                    <div className="flex items-center gap-1">
                      <div className="h-2 w-2 animate-pulse rounded-full bg-accent" />
                      <span className="text-xs text-accent">Active</span>
                    </div>
                  )}
                </label>
              );
            })}
          </div>
        </div>

        {broadcastSessions.size > 0 && (
          <div>
            <h3 className="mb-2 text-xs font-medium text-fg-muted">
              {strings.broadcast.sendCommand}
            </h3>
            <div className="flex gap-2">
              <input
                type="text"
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder={strings.broadcast.commandPlaceholder}
                className="flex-1 rounded-md border border-border bg-bg-sidebar px-3 py-2 text-sm text-fg placeholder:text-fg-muted focus:border-accent focus:outline-none"
              />
              <Button
                variant="primary"
                size="sm"
                onClick={handleSend}
                disabled={!input.trim()}
              >
                {strings.broadcast.send}
              </Button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
