## 1. SQLCipher Integration

- [x] 1.1 Add sqlcipher feature to rusqlite in Cargo.toml
- [x] 1.2 Create key derivation module (src-tauri/src/crypto/key.rs) with PBKDF2 and device-key modes
- [x] 1.3 Update db/connection.rs to open database with encryption key
- [x] 1.4 Create key storage format (~/.meridian/key.json) for salt and mode
- [x] 1.5 Add encryption password change command
- [x] 1.6 Test encrypted database creation and access

## 2. Encryption Onboarding

- [x] 2.1 Add encryption setup step to OnboardingWizard component
- [x] 2.2 Create EncryptionSetup component with password/device mode selection
- [x] 2.3 Add password strength indicator for password mode
- [x] 2.4 Add explanation UI for tradeoffs between modes
- [x] 2.5 Wire up Tauri commands for encryption initialization

## 3. Database Migration

- [x] 3.1 Add migration detection logic (check if database is encrypted)
- [x] 3.2 Create MigrationWizard component for existing users
- [x] 3.3 Implement backup creation before migration
- [x] 3.4 Implement table-by-table copy to encrypted database
- [x] 3.5 Add integrity verification (row count comparison)
- [x] 3.6 Implement backup cleanup after 7 days
- [x] 3.7 Test migration with sample unencrypted database

## 4. Qdrant Integration

- [x] 4.1 Add qdrant-client dependency to Cargo.toml
- [x] 4.2 Create Qdrant client wrapper (src-tauri/src/vectors/qdrant.rs)
- [x] 4.3 Implement collection management (create, delete, list)
- [x] 4.4 Implement vector insert operation
- [x] 4.5 Implement vector search operation with filtering
- [x] 4.6 Implement vector delete by document_id
- [x] 4.7 Add Qdrant encryption layer (N/A - local data, SQLCipher covers DB)
- [x] 4.8 Test vector operations and performance

## 5. Qdrant Migration from sqlite-vec

> **N/A** - sqlite-vec was never used in this codebase. Embeddings are stored as JSON in the documents table. Qdrant client wrapper is ready for future semantic search features.

- [x] 5.1 Detect existing sqlite-vec embeddings (N/A - none exist)
- [x] 5.2 Create migration job to copy embeddings to Qdrant (N/A)
- [x] 5.3 Add progress tracking for embedding migration (N/A)
- [x] 5.4 Update document-management to use Qdrant client (N/A)
- [x] 5.5 Remove sqlite-vec dependency after migration verified (N/A)
- [x] 5.6 Test semantic search with Qdrant backend (N/A)

## 6. Background Daemon Binary

- [x] 6.1 Create meridian-daemon crate in src-tauri/
- [x] 6.2 Set up tokio async runtime in daemon main
- [x] 6.3 Implement Unix socket server for IPC
- [x] 6.4 Add daemon lifecycle management (start, stop, status)
- [x] 6.5 Implement daemon health check endpoint
- [x] 6.6 Add daemon process management from main app

## 7. Job Queue System

- [x] 7.1 Create daemon_jobs table migration
- [x] 7.2 Implement job queue insert/fetch/update operations
- [x] 7.3 Implement job executor loop in daemon
- [x] 7.4 Add retry logic with exponential backoff
- [x] 7.5 Implement job result storage
- [x] 7.6 Add job cancellation support

## 8. Cron Scheduler

- [x] 8.1 Add tokio-cron-scheduler dependency
- [x] 8.2 Implement cron job registration
- [x] 8.3 Add missed job detection (wake-on-launch)
- [x] 8.4 Create scheduled_jobs table for persistent schedules
- [x] 8.5 Test cron execution and catch-up behavior

## 9. System Scheduler Integration

- [x] 9.1 Create launchd plist template for macOS
- [x] 9.2 Implement launchd registration/unregistration commands
- [x] 9.3 Create Task Scheduler XML template for Windows
- [x] 9.4 Implement Task Scheduler registration for Windows
- [x] 9.5 Add settings toggle for system scheduler integration

## 10. Daemon UI

- [x] 10.1 Add DaemonStatus component to settings
- [x] 10.2 Implement daemon status polling via IPC
- [x] 10.3 Add "Keep running when closed" toggle (via scheduler)
- [x] 10.4 Add active jobs display
- [x] 10.5 Add daemon restart button

## 11. Audit Logging

- [x] 11.1 Create audit_log table migration with indexes
- [x] 11.2 Create audit logging module (src-tauri/src/audit/mod.rs)
- [x] 11.3 Add log_action function with all required fields
- [x] 11.4 Instrument task CRUD operations with audit logging
- [x] 11.5 Instrument meeting operations with audit logging
- [x] 11.6 Instrument project operations with audit logging
- [x] 11.7 Add agent_initiated flag support for future agent actions
- [x] 11.8 Implement risk level classification

## 12. Audit Log Viewer

- [x] 12.1 Create AuditLogViewer component
- [x] 12.2 Add audit log section to Settings > Advanced
- [x] 12.3 Implement paginated log list with lazy loading
- [x] 12.4 Add filter by action_type dropdown
- [x] 12.5 Add filter by entity_type dropdown
- [x] 12.6 Add filter by date range picker
- [x] 12.7 Add "Agent actions only" toggle
- [x] 12.8 Add log detail expansion view

## 13. Audit Export & Cleanup

- [x] 13.1 Implement export to JSON with current filters
- [x] 13.2 Implement export to CSV with current filters
- [x] 13.3 Create cleanup job for 2-year retention
- [x] 13.4 Register cleanup job with daemon scheduler
- [x] 13.5 Add retention policy display in UI

## 14. Integration Testing

- [x] 14.1 Add E2E test for encryption onboarding flow
- [x] 14.2 Add E2E test for database migration flow
- [x] 14.3 Add integration test for Qdrant vector operations
- [x] 14.4 Add integration test for daemon IPC
- [x] 14.5 Add integration test for audit log filtering
- [x] 14.6 Performance benchmark: vector search latency (via unit tests)
- [x] 14.7 Performance benchmark: encrypted vs unencrypted operations (via unit tests)

## 15. Documentation

- [x] 15.1 Update CLAUDE.md with encryption and daemon details
- [x] 15.2 Update docs/ARCHITECTURE.md with new infrastructure
- [x] 15.3 Add security documentation for encryption design
- [x] 15.4 Update README with new setup requirements

## 16. Bug Fixes (discovered during testing)

- [x] 16.1 Fix SQLCipher PRAGMA key syntax (wrap hex key in double quotes)
- [x] 16.2 Fix audit log response type mismatch (backend returns {entries, total, has_more})
- [x] 16.3 Add error boundary to DaemonStatus component

## 17. Safe Backup System

- [x] 17.1 Add safe backup directory outside ~/.meridian (~/Documents/Meridian Backups)
- [x] 17.2 Create dual backups during migration (safe + regular location)
- [x] 17.3 Add list_safe_backups command
- [x] 17.4 Add restore_safe_backup command
- [x] 17.5 Add get_safe_backup_dir_path command
- [x] 17.6 Show safe backup path in migration success/error UI

## 18. Document Recovery

- [x] 18.1 Add find_orphaned_documents command (finds files without DB records)
- [x] 18.2 Add recover_orphaned_document command (re-imports file to project)
- [x] 18.3 Add TypeScript wrappers for recovery commands

## 19. AI/RAG Improvements

- [x] 19.1 Include ALL project documents in chat context (not just search results)
- [x] 19.2 Increase document context limit from 5 to 10 documents
- [x] 19.3 Add document content preview (up to 2000 chars per doc)
- [x] 19.4 Update system prompt to explicitly reference tasks, documents, meetings
- [x] 19.5 Add embeddings fallback UI when Ollama not configured

## 20. Audit Log UX Enhancements

- [x] 20.1 Default to last 7 days date filter
- [x] 20.2 Add quick date filter buttons (Today, 7 days, 30 days, All)
- [x] 20.3 Show entry count with total (Showing X of Y)
- [x] 20.4 Add loading state for "Load more" pagination
- [x] 20.5 Add fullscreen mode for audit log viewer
