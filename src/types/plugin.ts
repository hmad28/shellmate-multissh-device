export interface Plugin {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string | null;
  enabled: boolean;
  installedAt: string;
  updatedAt: string;
}

export interface PluginCapability {
  pluginId: string;
  capability: string;
  granted: boolean;
  config: string | null;
}

export interface PluginManifest {
  id: string;
  name: string;
  version: string;
  author: string;
  description?: string;
  apiVersion: string;
  capabilities: { name: string; config?: string }[];
  signaturePubkey?: string;
}
