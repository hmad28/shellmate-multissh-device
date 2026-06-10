import type { Host } from '@/types';

/**
 * Expand `{{variable}}` placeholders in a snippet command using the active host
 * context. Unknown placeholders are kept as-is so the user can spot mistakes.
 *
 * Supported variables (Phase 4):
 *   {{username}}       active host username
 *   {{host}}           hostname
 *   {{port}}           port
 *   {{label}}          host label
 *
 * Custom variables can be added by passing `extras`.
 */
export function expandSnippet(
  command: string,
  context: { host?: Host | null; extras?: Record<string, string> },
): string {
  const { host, extras } = context;

  return command.replace(
    /\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\}\}/g,
    (match, name: string) => {
      if (extras && Object.prototype.hasOwnProperty.call(extras, name)) {
        return extras[name] ?? match;
      }
      if (host) {
        switch (name) {
          case 'username':
            return host.username;
          case 'host':
          case 'hostname':
            return host.hostname;
          case 'port':
            return String(host.port);
          case 'label':
            return host.label;
        }
      }
      return match;
    },
  );
}

/** Returns the unique placeholder names found in a command, in order of first appearance. */
export function extractPlaceholders(command: string): string[] {
  const seen = new Set<string>();
  const out: string[] = [];
  for (const m of command.matchAll(/\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\}\}/g)) {
    const name = m[1];
    if (name && !seen.has(name)) {
      seen.add(name);
      out.push(name);
    }
  }
  return out;
}

const KNOWN = new Set(['username', 'host', 'hostname', 'port', 'label']);

/** Returns placeholders that are NOT a built-in (i.e., user must fill in). */
export function unknownPlaceholders(command: string): string[] {
  return extractPlaceholders(command).filter((p) => !KNOWN.has(p));
}
