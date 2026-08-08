# Sandboxed Execution Specification

## Overview

Execute skill scripts in isolated environments with configurable network access and resource limits.

## Execution Environment Selection

### Priority Order

1. **Docker** (preferred): Detected via `docker info` at runtime
2. **macOS sandbox-exec**: Built-in, use custom profile
3. **Linux firejail/bubblewrap**: Check availability at runtime
4. **Process isolation**: Fallback with restricted PATH and environment

### Detection Logic

```rust
pub enum SandboxBackend {
    Docker,
    MacOSSandbox,
    Firejail,
    Bubblewrap,
    ProcessIsolation,
}

pub fn detect_sandbox_backend() -> SandboxBackend {
    if is_docker_available() { return SandboxBackend::Docker; }
    if cfg!(target_os = "macos") { return SandboxBackend::MacOSSandbox; }
    if is_firejail_available() { return SandboxBackend::Firejail; }
    if is_bubblewrap_available() { return SandboxBackend::Bubblewrap; }
    SandboxBackend::ProcessIsolation
}
```

## Resource Limits

| Resource | Limit |
|----------|-------|
| Timeout | 60 seconds |
| Memory | 512 MB |
| CPU | 1 core |
| Processes | No fork (single process) |
| Disk | 100 MB temp space |

## Network Modes

### None (Default)
- No network access
- All outbound connections blocked

### Allowlist
- Skill declares allowed domains in frontmatter:
  ```yaml
  network:
    mode: allowlist
    domains:
      - api.github.com
      - api.openai.com
  ```
- Only listed domains permitted

### Full
- Unrestricted network access
- Requires explicit user opt-in with warning
- Re-approval required on any skill content change

## Docker Implementation

### Container Specification

```dockerfile
FROM python:3.11-slim
# Or node:20-slim, ruby:3.2-slim based on script extension

WORKDIR /skill
COPY . /skill/

# Resource limits applied via docker run
```

### Run Command

```bash
docker run --rm \
  --memory=512m \
  --cpus=1 \
  --pids-limit=10 \
  --network=none \  # or custom network for allowlist
  --read-only \
  --tmpfs /tmp:size=100m \
  -v /path/to/skill:/skill:ro \
  -v /path/to/output:/output:rw \
  skill-runner:latest \
  python /skill/script.py
```

## macOS sandbox-exec Profile

```scheme
(version 1)
(deny default)
(allow process-exec)
(allow file-read* (subpath "/skill"))
(allow file-write* (subpath "/output"))
(allow file-read* (subpath "/usr/lib"))
(allow file-read* (subpath "/System/Library"))
(deny network*)  ; or allow specific for allowlist
```

## File Output Management

### Output Directory Structure

```
~/.meridian/created_files/
├── 2026-08-05/
│   ├── report_143022.pdf
│   ├── analysis_143156.csv
│   └── ...
└── 2026-08-06/
    └── ...
```

### Naming Convention
- `{original_name}_{HHMMSS}.{ext}`
- Timestamp suffix prevents collisions

### Output Capture

After execution, scan `/output` directory for created files and:
1. Move to `~/.meridian/created_files/{date}/`
2. Record in `skill_outputs` table
3. Return file metadata to caller

## Trust Model

### Trust States

| State | Description |
|-------|-------------|
| `untrusted` | Default for new imports; requires approval before execution |
| `trusted` | User approved; can execute within declared permissions |
| `revoked` | Trust manually revoked or auto-revoked on content change |

### Auto-Revocation

Trust is automatically revoked when:
- `content_hash` changes after sync
- Network mode escalation requested
- Script files modified

### Trust UI

Settings → Skills → Trust Management:
- List all skills with trust state
- One-click revoke
- View permission history
