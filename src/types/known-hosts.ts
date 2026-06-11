export interface KnownHost {
  id: string;
  hostname: string;
  port: number;
  keyType: string;
  fingerprint: string;
  trusted: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface HostKeyVerificationResult {
  verified: boolean;
  isNewHost: boolean;
  storedFingerprint?: string;
  presentedFingerprint: string;
  keyType: string;
}

export interface VerifyHostKeyInput {
  hostname: string;
  port: number;
  keyType: string;
  publicKey: Uint8Array;
}

export interface TrustHostKeyInput {
  hostname: string;
  port: number;
  keyType: string;
  publicKey: Uint8Array;
}
