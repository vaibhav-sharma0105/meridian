# Security Architecture

This document describes Meridian's security model, encryption design, and best practices.

## Data Protection

### Database Encryption

Meridian uses **SQLCipher** to encrypt the SQLite database at rest with AES-256-CBC encryption.

#### Key Derivation Modes

**Device Mode** (default for new installs)
- Key derived from machine fingerprint: `PBKDF2(hostname + username + salt, 100,000 iterations)`
- Transparent to users — no password required
- Data is tied to the specific computer
- Cannot be moved to another machine without export/import

**Password Mode**
- Key derived from user password: `PBKDF2(password + salt, 100,000 iterations)`
- Portable across machines
- Password cannot be recovered if forgotten
- No password reset mechanism

#### Key Configuration

Key configuration is stored in `~/.meridian/key.json`:
```json
{
  "mode": "device",
  "salt": "hex-encoded-32-byte-salt",
  "version": 1
}
```

- The salt is randomly generated using a cryptographically secure RNG
- The derived key is never stored — only the salt and mode
- Key derivation uses the `ring` crate's PBKDF2 implementation

### Migration Path

Existing unencrypted databases continue to work (backward compatibility). Users can migrate to encryption via Settings > Advanced:

1. Backup is created automatically before migration
2. Data is copied to new encrypted database using SQLCipher's `sqlcipher_export()`
3. Original database is replaced after verification
4. Backup retained for 7 days

## Secret Storage

| Secret | Storage | Notes |
|--------|---------|-------|
| Zoom access/refresh tokens | OS keychain (keyring crate) | Encrypted by OS |
| AI provider API keys | OS keychain | Encrypted by OS |
| Sheets relay secret | `app_settings` table | Avoids unsigned app prompts |
| Database encryption key | Derived on-demand | Never stored |

## Audit Logging

All data mutations are logged to the `audit_log` table:

- **Action types**: create, update, delete, archive, sync, export
- **Risk classification**: low, medium, high, critical
- **Agent tracking**: Distinguishes user vs AI agent actions
- **Retention**: 2 years, automatically pruned

### Risk Levels

| Level | Triggers |
|-------|----------|
| Low | Create task, update task, sync |
| Medium | Delete task, update settings |
| High | Delete project, bulk operations |
| Critical | External API calls, data export |

## Background Daemon

The `meridian-daemon` process runs scheduled jobs independently:

- Communicates via Unix socket IPC (macOS) or named pipe (Windows)
- PID file at `~/.meridian/daemon.pid`
- Socket at `~/.meridian/daemon.sock`
- Can be registered with system scheduler (launchd/Task Scheduler)

### IPC Protocol

JSON-RPC over Unix socket:
```json
{"type": "status"}
{"type": "shutdown"}
{"type": "health"}
{"type": "run_job", "job_type": "sync_connections"}
```

## Vector Storage (Qdrant)

- Qdrant runs as an external service on `localhost:6334`
- Data directory at `~/.meridian/qdrant/`
- Vectors are project-scoped in separate collections
- Graceful degradation if Qdrant unavailable

## Best Practices

1. **Use password mode** for portable/shared machines
2. **Enable "Start at login"** to keep background sync running
3. **Export data regularly** if using device mode
4. **Review audit log** for unexpected agent actions
5. **Keep backups** before major operations

## Threat Model

### In Scope
- Data at rest (encrypted database)
- Secret storage (OS keychain)
- Audit trail (2-year retention)

### Out of Scope
- Data in transit to AI providers (HTTPS assumed)
- Physical access attacks
- Malicious code injection in app
- Qdrant data encryption (future work)
