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
      <div className="relative max-w-md w-full mx-4 rounded-lg bg-bg-elevated shadow-xl border border-border">
        <div className="p-6">
          <div className="flex items-start justify-between mb-4">
            <div>
              <h2 className="text-lg font-semibold text-fg">
                {isMismatch
                  ? strings.hostKeyVerification.mismatchTitle
                  : strings.hostKeyVerification.newHostTitle}
              </h2>
              <p className="text-sm text-fg-muted mt-1">
                {hostname}:{port}
              </p>
            </div>
            <button
              onClick={handleReject}
              disabled={loading}
              className="text-fg-muted hover:text-fg transition-colors"
            >
              <X size={20} />
            </button>
          </div>

          {isMismatch ? (
            <div className="mb-4 p-3 rounded-md bg-status-disconnected/10 border border-status-disconnected/30">
              <p className="text-sm text-status-disconnected font-medium mb-2">
                {strings.hostKeyVerification.mismatchWarning}
              </p>
              <div className="text-xs text-fg-muted space-y-1">
                <div>
                  <span className="font-medium">{strings.hostKeyVerification.stored}:</span>{' '}
                  {result.storedFingerprint || 'N/A'}
                </div>
                <div>
                  <span className="font-medium">{strings.hostKeyVerification.presented}:</span>{' '}
                  {result.presentedFingerprint}
                </div>
              </div>
            </div>
          ) : (
            <div className="mb-4 p-3 rounded-md bg-bg-sidebar border border-border-subtle">
              <p className="text-sm text-fg-muted mb-2">
                {strings.hostKeyVerification.newHostMessage}
              </p>
              <div className="text-xs text-fg-muted">
                <div>
                  <span className="font-medium">{strings.hostKeyVerification.fingerprint}:</span>{' '}
                  {result.presentedFingerprint}
                </div>
                <div>
                  <span className="font-medium">{strings.hostKeyVerification.keyType}:</span>{' '}
                  {result.keyType}
                </div>
              </div>
            </div>
          )}

          <div className="flex gap-2 justify-end">
            <Button
              variant="ghost"
              onClick={handleReject}
              disabled={loading}
            >
              {strings.hostKeyVerification.reject}
            </Button>
            {isMismatch ? (
              <Button
                variant="primary"
                onClick={handleTrust}
                disabled={loading}
                className="bg-status-disconnected hover:bg-status-disconnected/90"
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
