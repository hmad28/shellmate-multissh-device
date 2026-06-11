export type PortForwardType = 'local' | 'remote';

export interface PortForwardRule {
  id: string;
  sessionId: string;
  ruleType: PortForwardType;
  localPort: number;
  remoteHost: string;
  remotePort: number;
  enabled: boolean;
}

export interface PortForwardCreateInput {
  sessionId: string;
  ruleType: PortForwardType;
  localPort: number;
  remoteHost: string;
  remotePort: number;
}

export interface PortForwardListInput {
  sessionId: string;
}

export interface PortForwardIdInput {
  forwardId: string;
}
