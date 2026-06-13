export interface AuditEvent {
  id: string;
  hostId: string | null;
  eventType: string;
  payload: string;
  prevHash: string;
  createdAt: string;
}

export interface AuditSettings {
  hostId: string;
  auditEnabled: boolean;
  commandHistoryEnabled: boolean;
  redactionPatterns: string[] | null;
  retentionDays: number;
}

export interface AuditQuery {
  hostId?: string;
  eventType?: string;
  since?: string;
  until?: string;
  limit?: number;
}
