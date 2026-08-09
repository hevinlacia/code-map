# Configuration

Configuration is loaded from environment variables by `backend/src/config.rs`.

## Environment Variables

| Variable | Default | Description |
|---|---:|---|
| `CODE_MAP_HOST` | `127.0.0.1` | Backend bind host. |
| `CODE_MAP_PORT` | `18765` | Backend bind port. |
| `CODE_MAP_DATA_DIR` | `./data` | Local persisted settings, projects, and generated index cache directory. |
| `CODE_MAP_DEFAULT_WORKSPACE` | unset | Optional workspace path shown in the initial project list. |
| `CODE_MAP_FRONTEND_DIR` | `./frontend/dist` | Directory of built frontend assets served by the backend at `/`. |
| `CODE_MAP_ENABLE_WRITE_ACTIONS` | `false` | Enables managed local worktree creation under `CODE_MAP_DATA_DIR/worktrees/` for branch-name virtual views. |

`auto_refresh_enabled=true` is stored in runtime settings, not an environment
variable. When enabled with `indexing_enabled=true`, query and neighbors requests
run a default incremental scan before returning results for the selected project.

`CODE_MAP_ENABLE_WRITE_ACTIONS=true` allows `branch_name` queries to create
managed local worktrees under `CODE_MAP_DATA_DIR/worktrees/` when a requirement
branch exists but no existing worktree/overlay project is registered. Source
repositories are not deleted by Code Map; removing a project only removes the
index record and project-scoped cache files.

## Local Setup

```bash
cp .env.example .env
# edit .env as needed
```

The current backend does not automatically load `.env`; export variables in your shell or use a shell helper before starting the backend.

Example:

```bash
set -a
source .env
set +a
npm run api
```

## Ports

- Production (systemd service): `http://127.0.0.1:18765` (backend serves API and frontend).
- Dev frontend (Vite): `http://127.0.0.1:5178` (proxies `/api` and `/health` to the backend).

## Generated Data

Persisted settings, project registrations, and generated indexes stay under `CODE_MAP_DATA_DIR`, which is ignored by git. Do not place index cache files in source directories.

Current files:

- `settings.json` - runtime settings.
- `projects.json` - registered projects plus lightweight file index data and last scanned Git snapshots.
- `constants-{project_id}.json` - project-scoped resolved constant maps (rebuilt on full scan; reused on incremental scan).
- `worktrees/` - managed local worktrees created for branch-name virtual views when write actions are enabled.
- `notes.json` - verified agent notes.
