export interface VaultStatus {
  initialized: boolean;
  unlocked: boolean;
}

export interface ConnectByHostInput {
  hostId: string;
}

export type QuickConnectAuth =
  | { type: 'password'; password: string }
  | { type: 'key'; privateKey: string; passphrase: string | null };

export interface QuickConnectInput {
  hostname: string;
  port: number;
  username: string;
  label: string | null;
  auth: QuickConnectAuth;
}

export type SshSessionStatus =
  | 'connecting'
  | 'connected'
  | 'disconnected'
  | 'failed';

export interface SshStatusEvent {
  sessionId: string;
  status: SshSessionStatus;
  message: string | null;
}

export interface SshOutputEvent {
  sessionId: string;
  data: string;
}

export interface SshErrorEvent {
  sessionId: string;
  message: string;
}
