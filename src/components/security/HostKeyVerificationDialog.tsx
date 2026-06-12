import { useState } from 'react';
import { X } from 'lucide-react';
import { strings } from '@/i18n/en';
import { Button } from '@/components/ui/Button';
import type { HostKeyVerificationResult } from '@/types/known-hosts';

interface HostKeyVerificationDialogProps {
  hostname: string;
  port: number;
  result: HostKeyVerificationResult;
  onTrust: () => void;
  onReject: () => void;
}

export function HostKeyVerificationDialog({
  hostname,
  port,
  result,
  onTrust,
  onReject,
}: HostKeyVerificationDialogProps) {
  const [loading, setLoading] = useState(false);

  const handleTrust = async () => {
    setLoading(true);
    try {
      await onTrust();
    } finally {
      setLoading(false);
    }
  };

  const handleReject = async () => {
    setLoading(true);
    try {
      await onReject();
    } finally {
      setLoading(false);
    }
  };

  const isMismatch = !result.isNewHost && !result.verified;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="relative mx-4 w-full max-w-md rounded-lg border border-border bg-bg-elevated shadow-xl">
        <div className="p-6">
          <div className="mb-4 flex items-start justify-between">
            <div>
              <h2 className="text-lg font-semibold text-fg">
                {isMismatch
                  ? strings.hostKeyVerification.mismatchTitle
                  : strings.hostKeyVerification.newHostTitle}
              </h2>
              <p className="mt-1 text-sm text-fg-muted">
                {hostname}:{port}
              </p>
            </div>
            <button
              onClick={handleReject}
              disabled={loading}
              className="text-fg-muted transition-colors hover:text-fg"
            >
              <X size={20} />
            </button>
          </div>

          {isMismatch ? (
            <div className="bg-status-disconnected/10 border-status-disconnected/30 mb-4 rounded-md border p-3">
              <p className="mb-2 text-sm font-medium text-status-disconnected">
                {strings.hostKeyVerification.mismatchWarning}
              </p>
              <div className="space-y-1 text-xs text-fg-muted">
                <div>
                  <span className="font-medium">
                    {strings.hostKeyVerification.stored}:
                  </span>{' '}
                  {result.storedFingerprint || 'N/A'}
                </div>
                <div>
                  <span className="font-medium">
                    {strings.hostKeyVerification.presented}:
                  </span>{' '}
                  {result.presentedFingerprint}
                </div>
              </div>
            </div>
          ) : (
            <div className="mb-4 rounded-md border border-border-subtle bg-bg-sidebar p-3">
              <p className="mb-2 text-sm text-fg-muted">
                {strings.hostKeyVerification.newHostMessage}
              </p>
              <div className="text-xs text-fg-muted">
                <div>
                  <span className="font-medium">
                    {strings.hostKeyVerification.fingerprint}:
                  </span>{' '}
                  {result.presentedFingerprint}
                </div>
                <div>
                  <span className="font-medium">
                    {strings.hostKeyVerification.keyType}:
                  </span>{' '}
                  {result.keyType}
                </div>
              </div>
            </div>
          )}

          <div className="flex justify-end gap-2">
            <Button variant="ghost" onClick={handleReject} disabled={loading}>
              {strings.hostKeyVerification.reject}
            </Button>
            {isMismatch ? (
              <Button
                variant="primary"
                onClick={handleTrust}
                disabled={loading}
                className="hover:bg-status-disconnected/90 bg-status-disconnected"
              >
                {strings.hostKeyVerification.trustNew}
              </Button>
            ) : (
              <Button
                variant="primary"
                onClick={handleTrust}
                disabled={loading}
              >
                {strings.hostKeyVerification.trust}
              </Button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
