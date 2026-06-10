import { VaultGate } from '@/components/vault/VaultGate';
import { AppLayout } from '@/components/layout/AppLayout';

export default function App() {
  return (
    <VaultGate>
      <AppLayout />
    </VaultGate>
  );
}
