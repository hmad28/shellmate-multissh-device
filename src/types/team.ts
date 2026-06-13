export interface Team {
  id: string;
  name: string;
  createdAt: string;
}

export interface TeamMember {
  id: string;
  teamId: string;
  memberPubkey: string;
  memberLabel: string;
  addedAt: string;
  revokedAt: string | null;
}

export interface TeamShare {
  id: string;
  teamId: string;
  hostId: string;
  permission: 'read' | 'edit';
  sharedAt: string;
}

export interface CreateTeamInput {
  name: string;
}

export interface AddMemberInput {
  teamId: string;
  memberPubkey: string;
  memberLabel: string;
}

export interface ShareHostInput {
  teamId: string;
  hostId: string;
  permission: 'read' | 'edit';
}
