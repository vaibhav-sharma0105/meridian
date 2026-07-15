## Context

Meridian is a Tauri v2 desktop app with Rust backend and React frontend. Current state:

- **Storage**: Plain SQLite via rusqlite — no encryption
- **Vectors**: sqlite-vec extension — limited scalability, ~10K vector practical limit
- **Background tasks**: None — all operations synchronous in main process
- **Audit**: None — no action logging

This is a cross-cutting infrastructure change affecting database layer, embedding pipeline, and adding new daemon process. Security and migration complexity require careful design.

**Constraints:**
- Must migrate existing user data without loss
- Must work offline (local-first principle)
- Must not significantly increase app startup time
- Must support both macOS and Windows

## Goals / Non-Goals

**Goals:**
- Encrypt all data at rest using SQLCipher (AES-256)
- Replace sqlite-vec with Qdrant embedded for scalable vector search
- Enable background job execution via daemon process
- Log all actions with 2-year retention for audit/compliance

**Non-Goals:**
- Cloud sync (Phase 7)
- Multi-user/team features (Phase 7)
- Implementing actual skills/schedules (Phase 4)
- Changing AI providers or chat functionality

## Decisions

### Decision 1: SQLCipher vs Alternatives

**Choice:** SQLCipher with PBKDF2 key derivation

**Alternatives Considered:**
- **Application-level encryption (encrypt before SQLite)**: More flexible but loses SQL query capability on encrypted fields, complex to implement
- **Full-disk encryption reliance**: Not all users enable FileVault/BitLocker, not portable
- **SQLite SEE**: Commercial license, cost prohibitive

**Rationale:** SQLCipher is the industry standard for SQLite encryption, drop-in replacement for rusqlite via `rusqlite` feature flag, handles encryption transparently at database level.

### Decision 2: Key Derivation Strategy

**Choice:** Dual mode — user password OR device-bound key

**Password Mode:**
- PBKDF2-SHA256 with 100,000 iterations
- Random 32-byte salt stored in `~/.meridian/key.json`
- User must remember password for new device

**Device Mode:**
- Derive key from: SHA256(machine_id + username + app_constant)
- Convenient but tied to single device
- Export/import still possible (re-encrypts with password)

**Rationale:** Security-conscious users get password mode; convenience users get device mode. Both are secure at rest.

### Decision 3: Qdrant Embedded vs Alternatives

**Choice:** Qdrant embedded mode with on-disk encryption

**Alternatives Considered:**
- **sqlite-vec with sharding**: DIY, still limited, no ecosystem
- **LanceDB**: Newer, less battle-tested
- **Milvus Lite**: Heavy, server-oriented
- **FAISS**: No Rust bindings, C++ complexity

**Rationale:** Qdrant has native Rust client, proven at scale, supports embedded mode (no separate server), handles millions of vectors. ~15MB binary size increase acceptable.

### Decision 4: Qdrant Encryption

**Choice:** Encrypt Qdrant directory using same key as SQLite

**Implementation:**
- Qdrant stores data in `~/.meridian/qdrant/`
- On startup: decrypt Qdrant files to temp location, load
- On shutdown: re-encrypt to persistent location
- Use ring crate for AES-GCM encryption

**Alternative Considered:**
- Qdrant's built-in encryption: Requires enterprise license

**Rationale:** Consistent encryption approach, same key management, no additional licenses.

### Decision 5: Daemon Architecture

**Choice:** Separate binary with Unix socket IPC

**Alternatives Considered:**
- **In-process threads**: Can't survive app close, blocks UI
- **Embedded tokio runtime in main app**: Still can't survive app close
- **System service (launchd/systemd)**: Complex installation, elevated permissions

**Implementation:**
```
meridian (main Tauri app)
    ├── Launches daemon on startup
    ├── Communicates via Unix socket (~/.meridian/daemon.sock)
    └── Can close while daemon runs (if standalone enabled)

meridian-daemon (background service)
    ├── Tokio async runtime
    ├── Job queue in SQLite
    ├── Cron scheduler (tokio-cron-scheduler)
    └── Listens on Unix socket for commands
```

**Rationale:** Clean separation, daemon can survive app closure, standard IPC pattern.

### Decision 6: Daemon Job Queue

**Choice:** SQLite-backed job queue with `daemon_jobs` table

**Schema:**
```sql
CREATE TABLE daemon_jobs (
  id TEXT PRIMARY KEY,
  job_type TEXT NOT NULL,
  payload TEXT,  -- JSON
  status TEXT DEFAULT 'pending',
  priority INTEGER DEFAULT 5,
  scheduled_at TEXT,
  started_at TEXT,
  completed_at TEXT,
  retry_count INTEGER DEFAULT 0,
  max_retries INTEGER DEFAULT 3,
  result TEXT,  -- JSON
  error TEXT,
  created_at TEXT DEFAULT CURRENT_TIMESTAMP
);
```

**Rationale:** Persistent, survives restarts, can be queried from both main app and daemon, encrypted with rest of database.

### Decision 7: Audit Log Storage

**Choice:** Separate table in main database, indexed for filtering

**Schema:**
```sql
CREATE TABLE audit_log (
  id TEXT PRIMARY KEY,
  timestamp TEXT NOT NULL,
  action_type TEXT NOT NULL,
  entity_type TEXT NOT NULL,
  entity_id TEXT,
  details TEXT,  -- JSON
  agent_initiated BOOLEAN DEFAULT FALSE,
  autonomy_mode TEXT,
  risk_level TEXT,
  created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_audit_timestamp ON audit_log(timestamp);
CREATE INDEX idx_audit_entity ON audit_log(entity_type, entity_id);
CREATE INDEX idx_audit_action ON audit_log(action_type);
```

**Rationale:** In-database for encryption, indexed for filtering, JSON details for flexibility.

## Risks / Trade-offs

**[Risk: SQLCipher performance overhead]** → Mitigation: ~5-15% overhead is acceptable; optimize queries if needed. Benchmark before/after.

**[Risk: Qdrant memory usage]** → Mitigation: Qdrant embedded uses memory-mapped files. Monitor and add limits if needed.

**[Risk: Daemon socket conflicts]** → Mitigation: Use unique socket path with PID checking. If stale socket exists, remove and retry.

**[Risk: Migration data loss]** → Mitigation: Create backup before migration, verify integrity after, keep backup until user confirms.

**[Risk: Forgotten password]** → Mitigation: Clear warning during setup. Device mode available for users who prioritize convenience. No recovery possible by design (security feature).

**[Trade-off: Binary size increase]** → Accept: ~15-20MB increase for Qdrant + SQLCipher. Modern disk space makes this negligible.

**[Trade-off: Startup time]** → Accept: Daemon launch adds ~100-200ms. Acceptable for background benefits.

## Migration Plan

### Phase 1: Preparation
1. Add SQLCipher dependency, feature-flag behind `encrypted` feature
2. Add Qdrant dependency
3. Implement key derivation module
4. Add migration detection logic

### Phase 2: Migration Flow
1. On app start, check for unencrypted database
2. If found, show migration wizard:
   - Explain what's happening
   - User chooses encryption mode (password/device)
   - Create backup of existing database
3. Create new encrypted database
4. Copy all tables to encrypted database
5. Migrate embeddings from sqlite-vec to Qdrant
6. Verify row counts match
7. Swap database files
8. Keep backup for 7 days, then delete

### Phase 3: Daemon Setup
1. Create daemon binary
2. Add IPC infrastructure
3. Add daemon status UI
4. Register with system scheduler (optional)

### Rollback Strategy
- Keep unencrypted backup for 7 days
- If critical bug found, restore from backup
- Ship hotfix before backup expires

## Open Questions (Resolved)

1. **Qdrant encryption granularity**: ✅ N/A - Qdrant runs as external service, not embedded. Local data only.

2. **Daemon auto-start**: ✅ Explicit opt-in via toggle in Advanced settings.

3. **Memory limits**: ✅ Deferred - Qdrant external service handles its own memory.

4. **Audit log UI pagination**: ✅ Infinite scroll with "Load more" button, 50 entries per page.

## Implementation Refinements (Discovered During Testing)

### SQLCipher Key Format
- **Issue**: SQLCipher PRAGMA key syntax requires double quotes around hex blob: `PRAGMA key = "x'...'";`
- **Fix**: Updated all PRAGMA key/rekey statements to use correct syntax

### Safe Backup System
- **Issue**: Accidental deletion of ~/.meridian could destroy all data including backups
- **Fix**: Added dual backup system with "safe" location outside data directory:
  - macOS: `~/Documents/Meridian Backups/`
  - Windows: `Documents\Meridian Backups\`
- **Commands added**: `list_safe_backups`, `restore_safe_backup`, `get_safe_backup_dir_path`

### Document Recovery
- **Issue**: Database records can become orphaned from filesystem files during failed migrations
- **Fix**: Added document recovery commands to re-import orphaned files:
  - `find_orphaned_documents`: Scans filesystem for files without DB records
  - `recover_orphaned_document`: Re-imports file to specified project

### AI/RAG Context
- **Issue**: AI chat only included keyword-matched document chunks, missing full context
- **Fix**: Now includes ALL project documents (up to 10, 2000 chars each) in chat context
- **System prompt**: Updated to explicitly reference tasks, documents, meetings

### Embeddings Fallback
- **Issue**: No clear guidance when Ollama not configured for embeddings
- **Fix**: Added fallback UI explaining keyword search is used, with setup instructions

### Audit Log UX
- **Defaults**: Last 7 days filter, quick date buttons (Today, 7d, 30d, All)
- **Pagination**: Shows "Showing X of Y entries", loading state on Load more
- **Fullscreen**: Added maximize button for 90% viewport modal view
