import { useState } from 'react';
import { tauri } from '@/lib/tauri';
import { strings } from '@/i18n/en';
import { Shield, Key, Server, Check, Loader2 } from 'lucide-react';

export function VipAccessPanel() {
  const [status, setStatus] = useState<{
    hostExists: boolean;
    authorizedKeysInjected: boolean;
  } | null>(null);
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const checkStatus = async () => {
    try {
      const s = await tauri.vipAccess.getKeyStatus();
      setStatus(s);
    } catch (e) {
      setError(String(e));
    }
  };

  const generateAndInject = async () => {
    setLoading(true);
    setError(null);
    setResult(null);
    try {
      // Generate keypair
      const keyResult = await tauri.vipAccess.generateKeypair();
      const { publicKey, credentialId } = JSON.parse(keyResult);

      // Inject into authorized_keys
      await tauri.vipAccess.injectAuthorizedKeys(publicKey);

      // Create localhost host entry
      await tauri.vipAccess.createLocalhostHost(credentialId);

      setResult(
        `VIP access configured! Public key: ${publicKey.substring(0, 16)}...`,
      );
      await checkStatus();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-4 p-4">
      <div className="flex items-center gap-2 text-sm font-medium">
        <Shield className="h-4 w-4" />
        {strings.vipAccess?.title ?? 'VIP Passwordless Access'}
      </div>

      <p className="text-fg-muted text-xs">
        {strings.vipAccess?.description ??
          'Configure passwordless SSH access from your mobile device to this desktop.'}
      </p>

      {/* Status */}
      <button
        onClick={checkStatus}
        className="text-accent text-xs hover:underline"
      >
        {strings.vipAccess?.checkStatus ?? 'Check Status'}
      </button>

      {status && (
        <div className="space-y-2 text-xs">
          <div className="flex items-center gap-2">
            {status.authorizedKeysInjected ? (
              <Check className="h-3 w-3 text-green-500" />
            ) : (
              <Key className="text-fg-muted h-3 w-3" />
            )}
            <span>
              {strings.vipAccess?.authKeys ?? 'authorized_keys'}:{' '}
              {status.authorizedKeysInjected ? 'Injected' : 'Not found'}
            </span>
          </div>
          <div className="flex items-center gap-2">
            {status.hostExists ? (
              <Check className="h-3 w-3 text-green-500" />
            ) : (
              <Server className="text-fg-muted h-3 w-3" />
            )}
            <span>
              {strings.vipAccess?.localhostHost ?? 'VIP Host'}:{' '}
              {status.hostExists ? 'Created' : 'Not created'}
            </span>
          </div>
        </div>
      )}

      {/* Generate button */}
      <button
        onClick={generateAndInject}
        disabled={loading}
        className="bg-accent text-white hover:bg-accent-hover flex items-center gap-2 rounded-md px-3 py-2 text-xs font-medium disabled:opacity-50"
      >
        {loading ? (
          <Loader2 className="h-3 w-3 animate-spin" />
        ) : (
          <Key className="h-3 w-3" />
        )}
        {strings.vipAccess?.configure ?? 'Configure VIP Access'}
      </button>

      {result && (
        <div className="rounded bg-green-500/10 p-2 text-xs text-green-500">
          {result}
        </div>
      )}

      {error && (
        <div className="bg-red-500/10 text-red-400 rounded p-2 text-xs">
          {error}
        </div>
      )}
    </div>
  );
}
