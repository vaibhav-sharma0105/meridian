## Why

Meridian stores sensitive meeting transcripts, task data, and AI conversations on the user's machine with zero encryption. Before adding agentic capabilities (skills, autonomy, integrations), we must establish security and infrastructure foundations: encrypted storage, scalable vector search, background job execution, and audit logging. This is the prerequisite for all subsequent phases.

## What Changes

- **BREAKING**: Replace rusqlite with SQLCipher — existing databases require migration
- Add Qdrant embedded for scalable vector storage (replaces sqlite-vec)
- Introduce background daemon for scheduled jobs and async processing
- Add comprehensive audit logging with 2-year retention
- New onboarding step for encryption password (or device-key option)
- New settings panel for viewing audit logs

## Capabilities

### New Capabilities

- `encryption`: SQLCipher integration for always-encrypted database storage, key derivation, and migration from unencrypted databases
- `vector-storage`: Qdrant embedded integration for scalable vector similarity search, replacing sqlite-vec
- `background-daemon`: Tokio-based background service with IPC, system scheduler integration, and job queue management
- `audit-logging`: Comprehensive action logging with retention policy, UI viewer, and export capabilities

### Modified Capabilities

- `document-management`: Vector storage moves from sqlite-vec to Qdrant (internal change, same spec requirements)

## Impact

**Rust Backend:**
- `Cargo.toml`: Add sqlcipher, qdrant-client, tokio IPC dependencies
- `src-tauri/src/db/`: Rewrite connection handling for SQLCipher
- `src-tauri/src/daemon/`: New module for background service
- `src-tauri/src/ai/embeddings.rs`: Replace sqlite-vec with Qdrant client

**Frontend:**
- New onboarding step for encryption setup
- New audit log viewer in Settings > Advanced
- Daemon status indicator in UI

**Database:**
- All tables now encrypted at rest
- New `audit_log` table
- Qdrant data stored in `~/.meridian/qdrant/`

**Build:**
- Binary size increase (~15MB for Qdrant)
- New daemon binary or integrated service

**Migration:**
- Existing users: prompted to set encryption password on first launch
- Unencrypted data migrated to encrypted database
- sqlite-vec embeddings re-indexed into Qdrant
