import { useEffect } from 'react';
import { tauri } from '@/lib/tauri';
import { useVaultStore } from '@/stores/vault-store';

const POLL_INTERVAL_MS = 15_000;
const ACTIVITY_PING_INTERVAL_MS = 60_000;

/**
 * Periodic poll for backend idle auto-lock + activity ping.
 *
 * - Calls `vault_check_idle` on an interval. If backend reports idle-locked,
 *   refresh local state so VaultGate re-renders the unlock screen.
 * - Pings `vault_record_activity` while user is active so the idle counter
 *   resets. We use document interaction events as the activity signal.
 */
export function useAutoLock(): void {
  const unlocked = useVaultStore((s) => s.unlocked);
  const refresh = useVaultStore((s) => s.refresh);
  const recordActivity = useVaultStore((s) => s.recordActivity);

  // Idle poll: ask backend whether we should lock.
  useEffect(() => {
    if (!unlocked) return;
    const handle = window.setInterval(async () => {
      try {
        const wasLocked = await tauri.vault.checkIdle();
        if (wasLocked) {
          await refresh();
        }
      } catch {
        // ignore transient backend errors
      }
    }, POLL_INTERVAL_MS);
    return () => window.clearInterval(handle);
  }, [unlocked, refresh]);

  // Activity ping: any user input refreshes activity, but throttled.
  useEffect(() => {
    if (!unlocked) return;
    let last = 0;
    const ping = () => {
      const now = Date.now();
      if (now - last < ACTIVITY_PING_INTERVAL_MS) return;
      last = now;
      void recordActivity();
    };
    const events: (keyof DocumentEventMap)[] = [
      'mousedown',
      'keydown',
      'wheel',
      'touchstart',
    ];
    for (const e of events)
      document.addEventListener(e, ping, { passive: true });
    return () => {
      for (const e of events) document.removeEventListener(e, ping);
    };
  }, [unlocked, recordActivity]);
}
