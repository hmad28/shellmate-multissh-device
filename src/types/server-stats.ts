export interface ServerStats {
  hostname: string;
  osInfo: string;
  uptime: string;
  cpuCores: number;
  cpuUsage: number;
  cpuModel: string;
  memTotalMb: number;
  memUsedMb: number;
  memAvailableMb: number;
  memUsagePercent: number;
  disks: DiskInfo[];
  load1m: number;
  load5m: number;
  load15m: number;
  netRxMb: number;
  netTxMb: number;
  processes: ProcessInfo[];
}

export interface DiskInfo {
  filesystem: string;
  mount: string;
  totalGb: number;
  usedGb: number;
  usagePercent: number;
}

export interface ProcessInfo {
  pid: string;
  user: string;
  cpuPercent: number;
  memPercent: number;
  command: string;
}

export interface DockerContainer {
  id: string;
  name: string;
  image: string;
  status: string;
  ports: string;
}
