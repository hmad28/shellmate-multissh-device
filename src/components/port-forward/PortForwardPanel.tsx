import { useEffect, useState } from 'react';
import { usePortForwardStore } from '@/stores/port-forward-store';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Form';
import type { PortForwardType } from '@/types/port-forward';

interface PortForwardPanelProps {
  sessionId: string;
}

export function PortForwardPanel({ sessionId }: PortForwardPanelProps) {
  const {
    forwards,
    loadForwards,
    createForward,
    removeForward,
    toggleForward,
  } = usePortForwardStore();

  const [showForm, setShowForm] = useState(false);
  const [ruleType, setRuleType] = useState<PortForwardType>('local');
  const [localPort, setLocalPort] = useState('');
  const [remoteHost, setRemoteHost] = useState('localhost');
  const [remotePort, setRemotePort] = useState('');
  const [error, setError] = useState<string | null>(null);

  const sessionForwards = forwards[sessionId] || [];

  useEffect(() => {
    loadForwards(sessionId);
  }, [sessionId]);

  const handleCreate = async () => {
    setError(null);
    const local = parseInt(localPort);
    const remote = parseInt(remotePort);

    if (!local || local < 1 || local > 65535) {
      setError('Local port must be between 1 and 65535');
      return;
    }
    if (!remote || remote < 1 || remote > 65535) {
      setError('Remote port must be between 1 and 65535');
      return;
    }
    if (!remoteHost.trim()) {
      setError('Remote host is required');
      return;
    }

    try {
      await createForward(
        sessionId,
        ruleType,
        local,
        remoteHost.trim(),
        remote,
      );
      setShowForm(false);
      setLocalPort('');
      setRemoteHost('localhost');
      setRemotePort('');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create forward');
    }
  };

  const handleToggle = async (forwardId: string) => {
    await toggleForward(forwardId, sessionId);
  };

  const handleRemove = async (forwardId: string) => {
    if (confirm('Remove this port forward?')) {
      await removeForward(forwardId, sessionId);
    }
  };

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-semibold">Port Forwarding</h3>
        <Button
          variant="secondary"
          size="sm"
          onClick={() => setShowForm(!showForm)}
        >
          {showForm ? 'Cancel' : 'Add Rule'}
        </Button>
      </div>

      {showForm && (
        <div className="bg-muted/30 flex flex-col gap-3 rounded-lg border p-4">
          <div className="flex items-center gap-2">
            <label className="flex items-center gap-2">
              <input
                type="radio"
                name="ruleType"
                checked={ruleType === 'local'}
                onChange={() => setRuleType('local')}
                className="form-radio"
              />
              <span className="text-sm">Local (-L)</span>
            </label>
            <label className="flex items-center gap-2">
              <input
                type="radio"
                name="ruleType"
                checked={ruleType === 'remote'}
                onChange={() => setRuleType('remote')}
                className="form-radio"
              />
              <span className="text-sm">Remote (-R)</span>
            </label>
          </div>

          <div className="grid grid-cols-3 gap-2">
            <div>
              <label className="text-muted-foreground mb-1 block text-xs">
                Local Port
              </label>
              <Input
                type="number"
                value={localPort}
                onChange={(e) => setLocalPort(e.target.value)}
                placeholder="8080"
              />
            </div>
            <div>
              <label className="text-muted-foreground mb-1 block text-xs">
                Remote Host
              </label>
              <Input
                type="text"
                value={remoteHost}
                onChange={(e) => setRemoteHost(e.target.value)}
                placeholder="localhost"
              />
            </div>
            <div>
              <label className="text-muted-foreground mb-1 block text-xs">
                Remote Port
              </label>
              <Input
                type="number"
                value={remotePort}
                onChange={(e) => setRemotePort(e.target.value)}
                placeholder="80"
              />
            </div>
          </div>

          {error && <div className="text-destructive text-xs">{error}</div>}

          <div className="flex items-center gap-2">
            <Button size="sm" onClick={handleCreate}>
              Create
            </Button>
            <Button
              size="sm"
              variant="secondary"
              onClick={() => setShowForm(false)}
            >
              Cancel
            </Button>
          </div>
        </div>
      )}

      {sessionForwards.length === 0 ? (
        <div className="text-muted-foreground py-8 text-center text-sm">
          No port forwards configured
        </div>
      ) : (
        <div className="flex flex-col gap-2">
          {sessionForwards.map((forward) => (
            <div
              key={forward.id}
              className="flex items-center justify-between rounded-lg border p-3"
            >
              <div className="flex items-center gap-3">
                <input
                  type="checkbox"
                  checked={forward.enabled}
                  onChange={() => handleToggle(forward.id)}
                  className="form-checkbox"
                />
                <div className="flex flex-col">
                  <div className="text-sm font-medium">
                    {forward.ruleType === 'local' ? 'Local' : 'Remote'} Forward
                  </div>
                  <div className="text-muted-foreground text-xs">
                    {forward.ruleType === 'local'
                      ? `localhost:${forward.localPort} → ${forward.remoteHost}:${forward.remotePort}`
                      : `${forward.remoteHost}:${forward.remotePort} → localhost:${forward.localPort}`}
                  </div>
                </div>
              </div>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => handleRemove(forward.id)}
              >
                Remove
              </Button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
