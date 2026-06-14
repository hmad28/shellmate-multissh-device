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
