import { useState } from 'react';
import { Radio, X } from 'lucide-react';
import { strings } from '@/i18n/en';
import { cn } from '@/lib/cn';
import { Button } from '@/components/ui/Button';
import { useBroadcastStore } from '@/stores/broadcast-store';
import { useTabStore } from '@/stores/tab-store';

interface BroadcastModePanelProps {
  onClose: () => void;
}

export function BroadcastModePanel({ onClose }: BroadcastModePanelProps) {
  const { tabs } = useTabStore();
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
    <div className="flex flex-col h-full bg-bg">
      <div className="flex items-center justify-between px-4 py-3 border-b border-border">
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
          className="text-fg-muted hover:text-fg transition-colors"
        >
          <X size={16} />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-4">
        <div className="mb-4">
          <h3 className="text-xs font-medium text-fg-muted mb-2">
            {strings.broadcast.selectSessions}
          </h3>
          <div className="space-y-2">
            {tabs.map((tab) => {
              const isActive = isSessionActive(tab.id);
              return (
                <label
                  key={tab.id}
                  className={cn(
                    'flex items-center gap-3 p-2 rounded-md cursor-pointer transition-colors',
                    isActive
                      ? 'bg-accent/10 border border-accent/30'
                      : 'bg-bg-sidebar hover:bg-bg-elevated border border-transparent'
                  )}
                >
                  <input
                    type="checkbox"
                    checked={isActive}
                    onChange={() => toggleSession(tab.id)}
                    className="rounded border-border"
                  />
                  <div className="flex-1 min-w-0">
                    <div className="text-sm text-fg truncate">{tab.label}</div>
                    <div className="text-xs text-fg-muted truncate">
                      {tab.id}
                    </div>
                  </div>
                  {isActive && (
                    <div className="flex items-center gap-1">
                      <div className="w-2 h-2 rounded-full bg-accent animate-pulse" />
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
            <h3 className="text-xs font-medium text-fg-muted mb-2">
              {strings.broadcast.sendCommand}
            </h3>
            <div className="flex gap-2">
              <input
                type="text"
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder={strings.broadcast.commandPlaceholder}
                className="flex-1 px-3 py-2 text-sm bg-bg-sidebar border border-border rounded-md text-fg placeholder:text-fg-muted focus:outline-none focus:border-accent"
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
