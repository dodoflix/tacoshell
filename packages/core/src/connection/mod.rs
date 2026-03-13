// Protocol adapter traits and implementations
//
// Each protocol adapter implements ConnectionAdapter plus one or more
// capability traits (TerminalAdapter, FileTransferAdapter, KubernetesAdapter).
//
// Implementations live in sub-modules:
//   ssh.rs   — SSH (implements TerminalAdapter)
//   sftp.rs  — SFTP (implements FileTransferAdapter, built on top of SSH)
//   ftp.rs   — FTP/FTPS (implements FileTransferAdapter)
//   k8s.rs   — Kubernetes (implements KubernetesAdapter)
