export type ConnectionStatus = 'connected' | 'connecting' | 'disconnected';

export interface Tab {
  id: string;
  hostId: string | null;
  label: string;
  status: ConnectionStatus;
}
