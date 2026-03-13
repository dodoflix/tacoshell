# Tacoshell — GitHub Storage Engine

## 1. Overview

Tacoshell uses a BYOD (Bring Your Own Database) model. All user data is stored in a private GitHub repository named `tacoshell-vault` that the app creates automatically in the authenticated user's account. No Tacoshell-controlled servers store any user data.

---

## 2. Repository Structure

```
tacoshell-vault/           (private GitHub repository)
├── vault.json             # Encrypted vault blob
├── meta.json              # Unencrypted metadata
└── .tacoshell-marker      # Empty file identifying this as a Tacoshell vault
```

### vault.json (encrypted)

```json
{
  "schema_version": "1",
  "items": [
    {
      "id": "01JNMQR...",
      "nonce": "base64-encoded-96-bit-nonce",
      "ciphertext": "base64-encoded-encrypted-payload",
      "tag": "base64-encoded-16-byte-auth-tag",
      "version": "1",
      "created_at": "2026-03-13T00:00:00Z",
      "updated_at": "2026-03-13T00:00:00Z"
    }
  ]
}
```

Each item's decrypted payload is a typed JSON object (`ConnectionProfile`, `SshKey`, `Password`, or `KubeConfigItem`). See [`SECURITY.md`](SECURITY.md) for the encryption details.

### meta.json (unencrypted)

```json
{
  "schema_version": "1",
  "tacoshell_version": "0.1.0",
  "created_at": "2026-03-13T00:00:00Z",
  "last_sync": "2026-03-13T00:00:00Z"
}
```

---

## 3. Sync Protocol

### 3.1 Initial Load (App Launch)

```
1. GET /repos/{user}/tacoshell-vault/contents/vault.json
   (returns file content + SHA of current version)

2. Decode base64 content → parse vault.json

3. Decrypt each item with the Master Key

4. Store decrypted vault in memory (useVaultStore)

5. Store encrypted vault + SHA in local cache
   (IndexedDB on web, app_data on desktop/mobile)
```

If the request fails (network unavailable): fall back to local cache. The user is notified that they are working offline and changes will sync when connectivity is restored.

### 3.2 Writing (Profile Create / Update / Delete)

```
1. Apply change to in-memory vault (optimistic update)

2. Encrypt the full vault blob with the Master Key

3. PUT /repos/{user}/tacoshell-vault/contents/vault.json
   {
     message: "tacoshell: sync vault",
     content: base64(encrypted_vault_json),
     sha: <sha-from-last-fetch>   ← optimistic concurrency lock
   }

4a. 200 OK → update local cache with new SHA. Done.

4b. 422 (SHA mismatch — concurrent edit from another device):
    → Fetch remote vault
    → Merge (see §3.3)
    → Retry PUT with new SHA
```

### 3.3 Conflict Resolution

Conflicts occur when the same vault is modified on two devices before either syncs.

**Merge strategy**: per-item last-write-wins, using `updated_at` timestamps.

```
For each item ID that appears in both local and remote:
  if local.updated_at > remote.updated_at → keep local version
  if remote.updated_at > local.updated_at → keep remote version
  if equal timestamps → keep both, flag as conflict

Items only in local → add to merged vault
Items only in remote → add to merged vault (don't delete local deletes)

Exception: if an item was deleted locally (not present) and modified remotely
  (present with newer timestamp) → restore remote version and surface conflict UI
```

**True conflicts** (same item, same timestamp, different content) are surfaced in the UI as a side-by-side diff, letting the user choose which version to keep.

### 3.4 Sync Triggers

| Event | Action |
|-------|--------|
| App launch | Full fetch + decrypt |
| Profile create/update/delete | Encrypt + push |
| App close / backgrounded | Flush pending writes |
| Manual "Sync now" | Full fetch + merge + push |
| Network restored (was offline) | Full fetch + merge + push pending changes |

Sync is **lazy** — the app does not poll GitHub. The maximum sync frequency is bounded by user actions.

---

## 4. GitHub API Usage

### Required Scopes

| Scope | Purpose |
|-------|---------|
| `repo` | Create and write to the private `tacoshell-vault` repository |
| `read:user` | Retrieve the user's GitHub user ID (used as KDF salt) |

### API Calls Per Operation

| Operation | API Calls |
|-----------|-----------|
| Initial setup (create vault repo) | 2 (create repo + create vault.json) |
| App launch (load vault) | 1 (GET vault.json) |
| Save a change | 1 (PUT vault.json) |
| Conflict resolution | 2 (GET vault.json + PUT vault.json) |

**Rate limit**: GitHub allows 5,000 API requests/hour for OAuth apps. A typical user session uses 2–10 API calls. The rate limit is practically unreachable with this lazy sync design.

### Error Handling

| HTTP Status | Meaning | Handling |
|-------------|---------|----------|
| 200 | Success | Update local cache |
| 404 | Repo not found | Offer to create vault |
| 409 | Repo name conflict | Append suffix, retry |
| 422 | SHA mismatch | Fetch + merge + retry |
| 403 | Scope insufficient | Re-auth with correct scopes |
| 429 | Rate limited | Exponential backoff, notify user |
| 5xx | GitHub outage | Fall back to local cache, retry later |

---

## 5. Local Cache

The local cache stores the encrypted `vault.json` bytes and the last-known GitHub SHA. This allows:
- Offline operation: read vault without network
- Faster startup: compare SHA before fetching (skip fetch if unchanged)
- Pending writes: queue writes when offline

### Cache Locations

| Platform | Location |
|----------|----------|
| macOS | `~/Library/Application Support/tacoshell/cache/` |
| Windows | `%APPDATA%\tacoshell\cache\` |
| Linux | `~/.local/share/tacoshell/cache/` |
| Web (IndexedDB) | Origin-scoped IndexedDB database `tacoshell-cache` |

The cache stores the encrypted vault (already AES-256-GCM encrypted with the Master Key). No additional encryption layer is applied on desktop/mobile, as the OS file system encryption and app sandboxing provide sufficient protection. On web, the cache is additionally encrypted with a session key derived from a device-specific secret stored in the browser's credential store.

---

## 6. First-Time Setup Flow

```
1. User completes GitHub OAuth

2. App fetches user profile to get github_user_id

3. App checks if tacoshell-vault repo exists:
   GET /repos/{user}/tacoshell-vault

4a. Repo exists:
    → Load vault (standard launch flow)

4b. Repo does not exist:
    → Show "Create your vault" onboarding screen
    → User sets vault passphrase
    → App creates private repo: POST /user/repos { name: "tacoshell-vault", private: true }
    → App creates vault.json with empty item list (encrypted)
    → App creates meta.json
    → App creates .tacoshell-marker
    → Redirect to main app
```

---

## 7. Vault Migration

When the vault schema version changes (e.g., v1 → v2):

1. App detects `schema_version` mismatch on load
2. Decrypts all items using the old schema parser
3. Re-serializes items in the new schema format
4. Re-encrypts and pushes the new vault
5. Updates `meta.json` with the new `schema_version`

Migrations are always forward-only and tested with the full vault corpus as part of the CI integration test suite.
