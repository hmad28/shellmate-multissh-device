import { useUiStore } from '@/stores/ui-store';
import { VaultGate } from '@/components/vault/VaultGate';
import { Sidebar } from './Sidebar';
import { ContentArea } from './ContentArea';
import { StatusBar } from './StatusBar';
import { BottomNav } from './BottomNav';

function MobileContent() {
  const { activePanel } = useUiStore();

  return (
    <div className="flex h-[100dvh] flex-col bg-[var(--color-bg)] text-[var(--color-text)]">
      <div className="flex-1 overflow-hidden">
        {activePanel === 'hosts' ? (
          <Sidebar />
        ) : (
          <ContentArea />
        )}
      </div>
      <StatusBar />
      <BottomNav />
    </div>
  );
}

export function MobileLayout() {
  return (
    <VaultGate>
      <MobileContent />
    </VaultGate>
  );
}
