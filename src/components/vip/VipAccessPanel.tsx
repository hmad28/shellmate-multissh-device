import { useState } from 'react';
import { tauri } from '@/lib/tauri';
import { strings } from '@/i18n/en';
import { Shield, Key, Server, Check, Loader2 } from 'lucide-react';

export function VipAccessPanel() {
  const [status, setStatus] = useState<{
    hostExists: boolean;
    adminHostExists: boolean;
    authorizedKeysInjected: boolean;
    adminKeysInjected: boolean;
  } | null>(null);
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [username, setUsername] = useState('');
  const [asAdmin, setAsAdmin] = useState(false);

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

      // Inject into authorized_keys (optionally as admin)
      await tauri.vipAccess.injectAuthorizedKeys(publicKey, asAdmin);

      // Create localhost host entry
      await tauri.vipAccess.createLocalhostHost(
        credentialId,
        undefined,
        username || undefined,
        asAdmin,
      );

      setResult(
        `VIP access configured successfully! ${
          asAdmin ? 'Administrator keys' : 'User keys'
        } injected.`,
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
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 bg-bg-elevated/20 p-3 rounded-lg border border-[var(--color-border)] max-w-xl text-xs">
          <div className="space-y-2">
            <h4 className="font-semibold text-[10px] uppercase tracking-wider text-fg-muted">Standard VIP</h4>
            <div className="flex items-center gap-2">
              {status.authorizedKeysInjected ? (
                <Check className="h-3 w-3 text-green-500" />
              ) : (
                <Key className="text-fg-muted h-3 w-3" />
              )}
              <span>User Keys: {status.authorizedKeysInjected ? 'Configured' : 'Not found'}</span>
            </div>
            <div className="flex items-center gap-2">
              {status.hostExists ? (
                <Check className="h-3 w-3 text-green-500" />
              ) : (
                <Server className="text-fg-muted h-3 w-3" />
              )}
              <span>Local Host Entry: {status.hostExists ? 'Created' : 'Not created'}</span>
            </div>
          </div>

          <div className="space-y-2">
            <h4 className="font-semibold text-[10px] uppercase tracking-wider text-fg-muted flex items-center gap-1 text-[var(--color-accent)]">
              <Shield className="h-3 w-3" /> Administrator VIP
            </h4>
            <div className="flex items-center gap-2">
              {status.adminKeysInjected ? (
                <Check className="h-3 w-3 text-green-500" />
              ) : (
                <Key className="text-fg-muted h-3 w-3" />
              )}
              <span>Admin Keys: {status.adminKeysInjected ? 'Configured' : 'Not found'}</span>
            </div>
            <div className="flex items-center gap-2">
              {status.adminHostExists ? (
                <Check className="h-3 w-3 text-green-500" />
              ) : (
                <Server className="text-fg-muted h-3 w-3" />
              )}
              <span>Admin Host Entry: {status.adminHostExists ? 'Created' : 'Not created'}</span>
            </div>
          </div>
        </div>
      )}

      {/* Username input */}
      <div className="flex flex-col gap-1 max-w-xs">
        <label className="text-fg-muted text-[10px] uppercase tracking-wider font-semibold">
          SSH Username (Optional)
        </label>
        <input
          type="text"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          placeholder="e.g. Administrator, root, or leave empty"
          className="bg-bg-elevated border border-[var(--color-border)] rounded px-2 py-1.5 text-xs text-[var(--color-fg)] focus:outline-none focus:border-[var(--color-accent)] transition-colors"
        />
      </div>

      {/* Run as Administrator toggle */}
      <div className="flex items-start gap-3 bg-bg-elevated/40 border border-[var(--color-border)] rounded-lg p-3 max-w-sm">
        <input
          type="checkbox"
          id="asAdminToggle"
          checked={asAdmin}
          onChange={(e) => setAsAdmin(e.target.checked)}
          className="mt-0.5 rounded border-[var(--color-border)] text-[var(--color-accent)] focus:ring-[var(--color-accent)]"
        />
        <div className="flex flex-col gap-0.5">
          <label htmlFor="asAdminToggle" className="text-xs font-semibold select-none cursor-pointer flex items-center gap-1.5 text-[var(--color-fg)]">
            <Shield className="h-3 w-3 text-[var(--color-accent)]" />
            Enable Administrator Privileges
          </label>
          <span className="text-[10px] text-fg-muted leading-normal">
            Windows: Key will be authorized in the System administrators file (triggers a UAC popup).
            macOS/Linux: Key will be authorized for the root user.
          </span>
        </div>
      </div>

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
        <div className="rounded-lg bg-green-500/10 border border-green-500/20 p-3 text-xs text-green-400 space-y-2">
          <div className="font-semibold flex items-center gap-1.5">
            <Check className="h-4 w-4 text-green-500" />
            {result}
          </div>
          <div className="border-t border-green-500/10 pt-2 space-y-1.5 text-[11px] text-[var(--color-fg-muted)] leading-relaxed">
            <p className="font-semibold text-[var(--color-fg)]">Next Steps & How to Use:</p>
            <ol className="list-decimal pl-4 space-y-1">
              <li>
                <strong>Cloud Sync:</strong> Ensure E2EE Cloud Sync is active on both this desktop and your phone. This copies the private key and localhost configuration to your mobile device.
              </li>
              <li>
                <strong>Local Network:</strong> Make sure your phone and desktop are on the same Wi-Fi network (or connected via a VPN like Tailscale).
              </li>
              <li>
                <strong>Connect:</strong> Open the app on your phone. You will see <span className="text-[var(--color-accent)] font-semibold">"VIP Localhost"</span> (or <span className="text-[var(--color-accent)] font-semibold">"VIP Localhost (Admin)"</span>) in your Hosts list. Tap it to launch the terminal!
              </li>
            </ol>
            <p className="mt-1 text-[10px] text-fg-muted bg-bg-elevated/40 p-1.5 rounded border border-[var(--color-border)]">
              💡 <em>Note:</em> Make sure your desktop's OpenSSH Server is running. If you haven't started it yet, refer to the <a href="file:///C:/Users/Pongo/.gemini/antigravity-cli/brain/ac930051-3258-4ee8-9d72-6261b2fb8265/vip_administrator_access_guide.md" className="text-[var(--color-accent)] underline">VIP Access Guide</a> for easy setup commands.
            </p>
          </div>
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
