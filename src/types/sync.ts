export interface SyncStatus {
  configured: boolean;
  enabled: boolean;
  backendType: string | null;
  endpointUrl: string | null;
  lastSyncAt: string | null;
  pendingChanges: number;
  syncedEntities: number;
}

export interface SyncConfigureInput {
  backendType: string;
  endpointUrl: string;
  credentials: string;
}

export interface SyncResult {
  uploaded: number;
  downloaded: number;
  conflicts: number;
}
