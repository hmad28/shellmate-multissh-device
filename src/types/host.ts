export type AuthType = 'password' | 'key' | 'key_passphrase';

export interface Host {
  id: string;
  label: string;
  hostname: string;
  port: number;
  username: string;
  authType: AuthType;
  credentialId: string;
  groupId: string | null;
  tags: string[];
  notes: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface HostInput {
  label: string;
  hostname: string;
  port: number;
  username: string;
  authType: AuthType;
  credentialId: string;
  groupId: string | null;
  tags: string[];
  notes: string | null;
}

export interface Group {
  id: string;
  name: string;
  color: string | null;
  parentId: string | null;
  sortOrder: number;
}

export interface GroupInput {
  name: string;
  color: string | null;
  parentId: string | null;
  sortOrder: number | null;
}
