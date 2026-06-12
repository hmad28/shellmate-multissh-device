import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { tauri } from '@/lib/tauri';
import { useSshStore } from '@/stores/ssh-store';
import { useTabStore } from '@/stores/tab-store';
import { useUiStore } from '@/stores/ui-store';
import { HostKeyVerificationDialog } from '@/components/security/HostKeyVerificationDialog';
import { Sidebar } from './Sidebar';
import { TabBar } from './TabBar';
import { TitleBar } from './TitleBar';
import { StatusBar } from './StatusBar';
import { ContentArea } from './ContentArea';

interface PendingVerification {
  sessionId: string;
  hostname: string;
  port: number;
  keyType: string;
  fingerprint: string;
  publicKey: Uint8Array;
  isNewHost: boolean;
  storedFingerprint?: string;
}

export function AppLayout() {
  const [pendingVerification, setPendingVerification] =
    useState<PendingVerification | null>(null);
  const toggleSidebar = useUiStore((s) => s.toggleSidebar);

  // Ctrl+B to toggle sidebar
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'b') {
        e.preventDefault();
        toggleSidebar();
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [toggleSidebar]);

  useEffect(() => {
    const unlisten = listen<any>('ssh:host-key-verification', (event) => {
      const {
        sessionId,
        hostname,
        port,
        keyType,
        fingerprint,
        publicKey,
        isNewHost,
        storedFingerprint,
      } = event.payload;

      setPendingVerification({
        sessionId,
        hostname,
        port,
        keyType,
        fingerprint,
        publicKey: new Uint8Array(publicKey),
        isNewHost,
        storedFingerprint,
      });
    });

    return () => {
      void unlisten.then((fn) => fn());
    };
  }, []);

  const handleTrust = async () => {
    if (!pendingVerification) return;
    try {
      await tauri.knownHosts.trust({
        hostname: pendingVerification.hostname,
        port: pendingVerification.port,
        keyType: pendingVerification.keyType,
        publicKey: pendingVerification.publicKey,
      });
      void useSshStore.getState().retryAttempt(pendingVerification.sessionId);
    } catch (err) {
      console.error('Failed to trust host key', err);
    } finally {
      setPendingVerification(null);
    }
  };

  const handleReject = () => {
    if (!pendingVerification) return;
    const attempt =
      useSshStore.getState().pendingAttempts[pendingVerification.sessionId];
    if (attempt) {
      useTabStore.getState().updateTabStatus(attempt.tabId, 'disconnected');
      useSshStore.getState().removeAttempt(pendingVerification.sessionId);
    }
    setPendingVerification(null);
  };

  return (
    <div className="flex h-full w-full flex-col overflow-hidden bg-bg text-fg">
      <TitleBar />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar />
        <div className="flex flex-1 flex-col overflow-hidden">
          <TabBar />
          <ContentArea />
        </div>
      </div>
      <StatusBar />

      {pendingVerification && (
        <HostKeyVerificationDialog
          hostname={pendingVerification.hostname}
          port={pendingVerification.port}
          result={{
            verified: false,
            isNewHost: pendingVerification.isNewHost,
            ...(pendingVerification.storedFingerprint
              ? { storedFingerprint: pendingVerification.storedFingerprint }
              : {}),
            presentedFingerprint: pendingVerification.fingerprint,
            keyType: pendingVerification.keyType,
          }}
          onTrust={handleTrust}
          onReject={handleReject}
        />
      )}
    </div>
  );
}
