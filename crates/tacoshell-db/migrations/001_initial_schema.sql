-- Initial database schema for Tacoshell
-- Secrets table
CREATE TABLE IF NOT EXISTS secrets (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('password', 'private_key', 'token', 'kubeconfig')),
    encrypted_value BLOB NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_secrets_name ON secrets(name);

-- Servers table
CREATE TABLE IF NOT EXISTS servers (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    host TEXT NOT NULL,
    port INTEGER NOT NULL DEFAULT 22,
    username TEXT NOT NULL,
    protocol TEXT NOT NULL DEFAULT 'ssh' CHECK (protocol IN ('ssh', 'sftp', 'ftp')),
    tags TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_servers_name ON servers(name);
CREATE INDEX IF NOT EXISTS idx_servers_host ON servers(host);

-- Server-Secret junction table (many-to-many)
CREATE TABLE IF NOT EXISTS server_secrets (
    server_id TEXT NOT NULL,
    secret_id TEXT NOT NULL,
    priority INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (server_id, secret_id),
    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE,
    FOREIGN KEY (secret_id) REFERENCES secrets(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_server_secrets_server ON server_secrets(server_id);
CREATE INDEX IF NOT EXISTS idx_server_secrets_secret ON server_secrets(secret_id);

