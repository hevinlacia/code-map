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
| `CODE_MAP_ENABLE_WRITE_ACTIONS` | `false` | Enables settings that may later trigger filesystem writes or refresh tasks. |

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
- `projects.json` - registered projects plus lightweight file index data.
- `constants.json` - resolved constant map (rebuilt on full scan; reused on incremental scan).
- `notes.json` - verified agent notes.
