export interface SessionRecording {
  id: string;
  sessionId: string;
  hostId: string | null;
  hostLabel: string;
  startedAt: string;
  endedAt: string | null;
  durationSecs: number | null;
  eventCount: number;
}

export interface SessionEvent {
  timestampMs: number;
  eventType: string;
  data: string;
}

export interface SshKey {
  id: string;
  name: string;
  keyType: string;
  fingerprint: string;
  publicKey: string;
  hasPassphrase: boolean;
  createdAt: string;
}

export interface LocalSession {
  id: string;
  shell: string;
  pid: number;
}

export interface SftpTransfer {
  id: string;
  sourceHost: string;
  sourcePath: string;
  destHost: string;
  destPath: string;
  status: string;
  bytesTransferred: number;
}

export interface ParsedHost {
  label: string;
  hostname: string;
  port: number;
  username: string;
  authType: string;
  identityFile: string | null;
  proxyJump: string | null;
  forwardAgent: boolean;
  compression: boolean;
}

export interface ConnectionDiagnostics {
  hostname: string;
  port: number;
  dnsResolved: boolean;
  dnsIp: string | null;
  tcpConnected: boolean;
  tcpLatencyMs: number | null;
  sshBanner: string | null;
  hostKeyVerified: boolean;
  authMethod: string;
  authSuccess: boolean;
  ptyAllocated: boolean;
  errorStage: string | null;
  errorMessage: string | null;
}
