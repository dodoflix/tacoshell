# Tacoshell — Security Model

## 1. Threat Model

| Threat | Mitigation |
|--------|-----------|
| GitHub storage is compromised | All data is AES-256-GCM encrypted before leaving the device |
| WebSocket proxy (web mode) is compromised | Proxy only forwards encrypted SSH/TLS frames; it never decrypts |
| Device is lost/stolen | Vault requires passphrase (or biometric) on every session start |
| Master Key is leaked from memory | `zeroize` crate wipes sensitive memory on drop; key never touches disk |
| OAuth token is stolen | Token stored in OS keychain (desktop/mobile) or encrypted IndexedDB (web); revocation via GitHub |
| Vault sync conflict overwrites data | Per-profile last-write-wins with conflict UI; GitHub SHA-based optimistic locking |

**Explicit non-goals**: Server-side security (no server exists), protection against a fully compromised OS, DRM.

---

## 2. Key Derivation

```
User enters vault passphrase
          │
          ▼
Argon2id(
  password: passphrase_bytes,
  salt:     SHA-256(github_user_id_string),  // deterministic, never stored
  m_cost:   65536,   // 64 MB memory
  t_cost:   3,       // 3 iterations
  p_cost:   1,       // 1 lane
  output:   32 bytes // 256-bit Master Key
)
          │
          ▼
Master Key (256 bits) — held only in process memory
```

**Why Argon2id?**
- Memory-hard: defeats GPU/ASIC brute-force attacks
- Argon2id (hybrid) protects against both side-channel and GPU attacks
- Winner of the Password Hashing Competition (2015)
- Recommended by OWASP and NIST

**The salt** is derived deterministically from the GitHub user ID so:
- No salt needs to be stored or transmitted
- The same passphrase always produces the same Master Key on any device
- An attacker needs both the passphrase AND the GitHub user ID to derive the key

---

## 3. Encryption Model

Every sensitive vault item is individually encrypted:

```
┌─────────────────────────────────────────────┐
│ Vault Item (ConnectionProfile, Credential)   │
│                                              │
│  plaintext = JSON-serialized item            │
│  nonce     = random 96-bit value             │
│                                              │
│  ciphertext, tag = AES-256-GCM(             │
│      key:   Master Key,                      │
│      nonce: nonce,                           │
│      aad:   item_id + schema_version,        │
│      msg:   plaintext                        │
│  )                                           │
│                                              │
│  Stored envelope: {                          │
│      id:        uuid-v4,                     │
│      nonce:     base64(nonce),               │
│      ciphertext: base64(ciphertext),         │
│      tag:       base64(tag),                 │
│      version:   "1",                         │
│      created_at: ISO-8601,                   │
│      updated_at: ISO-8601                    │
│  }                                           │
└─────────────────────────────────────────────┘
```

**Why AES-256-GCM?**
- Hardware-accelerated (AES-NI) on all modern CPUs and mobile SoCs
- Provides both confidentiality and integrity (AEAD)
- 96-bit nonce: with random generation, collision probability is negligible for vault sizes

**Additional Authenticated Data (AAD)**: The `item_id + schema_version` is bound to the ciphertext via GCM authentication. This prevents an attacker from copying a ciphertext from one item to another without detection.

---

## 4. Vault File Format

The `vault.json` file stored in the GitHub repository has this structure:

```json
{
  "schema_version": "1",
  "items": [
    {
      "id": "01J...",
      "nonce": "base64...",
      "ciphertext": "base64...",
      "tag": "base64...",
      "version": "1",
      "created_at": "2026-03-13T00:00:00Z",
      "updated_at": "2026-03-13T00:00:00Z"
    }
  ]
}
```

The plaintext of each item (once decrypted) is a typed JSON object — one of:
- `ConnectionProfile`: host, port, username, protocol, auth method, display name
- `SshKey`: key type, PEM/OpenSSH private key material, associated profile IDs
- `Password`: username, password, associated profile IDs
- `KubeConfig`: cluster, context, auth section from a kubeconfig file

The `meta.json` file (unencrypted) contains only:
```json
{
  "schema_version": "1",
  "created_at": "2026-03-13T00:00:00Z",
  "last_sync": "2026-03-13T00:00:00Z",
  "tacoshell_version": "0.1.0"
}
```

---

## 5. OAuth Token Storage

| Platform | Storage Location |
|----------|----------------|
| Desktop (macOS) | macOS Keychain (via `keyring` crate) |
| Desktop (Windows) | Windows Credential Manager (via `keyring` crate) |
| Desktop (Linux) | libsecret / KWallet (via `keyring` crate) |
| Mobile (iOS) | iOS Keychain |
| Mobile (Android) | Android Keystore |
| Web | IndexedDB, encrypted with a session-derived key; cleared on tab close |

---

## 6. Optional Biometric Unlock (Desktop / Mobile)

When the user opts in to biometric unlock:

1. On first setup, the Master Key is encrypted with a platform key stored in the Secure Enclave / TPM / OS keychain, protected by biometric authentication.
2. On subsequent logins, the app requests biometric authentication. On success, the platform key is released and the Master Key is decrypted.
3. The vault passphrase is still required as a fallback (e.g., if biometric fails 5 times).

This is opt-in and clearly documented as a convenience tradeoff (the Master Key is persisted, albeit protected by hardware).

---

## 7. WebSocket Proxy Security (Web Mode)

The `ws-proxy` binary:
- Accepts WebSocket connections from the browser
- Opens a TCP connection to the remote server
- Forwards raw bytes in both directions
- Has **zero knowledge** of the protocol — it does not parse SSH, SFTP, or FTP frames
- The SSH/TLS handshake happens end-to-end between the browser (WASM) and the remote server
- The proxy cannot decrypt or tamper with data without detection (SSH MAC / TLS authentication)

**Deployment**: Users self-host the proxy (Docker image provided). An optional hosted instance may be offered in future as an opt-in convenience feature, with clear disclosure.

---

## 8. GitHub API Permissions

Tacoshell requests the minimum GitHub OAuth scopes required:

| Scope | Purpose |
|-------|---------|
| `repo` | Create and write to the private `tacoshell-vault` repository |
| `read:user` | Retrieve the user's GitHub ID (used as KDF salt) |

No other scopes are requested. Users can audit and revoke the OAuth app in GitHub Settings → Applications at any time.

---

## 9. Dependency Audit

Security-sensitive Rust crates:

| Crate | Purpose | Audit |
|-------|---------|-------|
| `ring` | Cryptographic primitives | Widely audited, used by rustls, AWS |
| `aes-gcm` | AES-GCM implementation | RustCrypto, actively maintained |
| `argon2` | Argon2id KDF | RustCrypto, matches reference implementation |
| `zeroize` | Memory wiping | RustCrypto, standard in security-sensitive Rust |
| `russh` | SSH protocol | Actively maintained pure-Rust SSH2 |
| `keyring` | OS secret storage | Wraps OS APIs, minimal attack surface |

CI runs `cargo audit` on every pull request to catch known vulnerabilities in the dependency tree.

---

## 10. Reporting Vulnerabilities

Please **do not** file public GitHub issues for security vulnerabilities. Instead, email `security@tacoshell.dev` with:
- Description of the vulnerability
- Steps to reproduce
- Potential impact assessment

We aim to respond within 48 hours and will credit researchers in release notes (unless anonymity is preferred).
