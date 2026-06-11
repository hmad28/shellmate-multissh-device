export interface SftpFile {
  name: string;
  size: number;
  permissions: number;
  modified: number;
  isDir: boolean;
  isSymlink: boolean;
}

export interface SftpOpenInput {
  sessionId: string;
}

export interface SftpListInput {
  sftpId: string;
  path?: string;
}

export interface SftpUploadInput {
  sftpId: string;
  localPath: string;
  remotePath: string;
}

export interface SftpDownloadInput {
  sftpId: string;
  remotePath: string;
  localPath: string;
}

export interface SftpRenameInput {
  sftpId: string;
  oldPath: string;
  newPath: string;
}

export interface SftpPathInput {
  sftpId: string;
  path: string;
}

export interface SftpCloseInput {
  sftpId: string;
}

export interface SftpProgressEvent {
  transferId: string;
  bytesTransferred: number;
  totalBytes: number;
  filename: string;
}
