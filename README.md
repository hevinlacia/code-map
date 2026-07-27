# Code Map

Agent-oriented repository map for reducing codebase exploration time and token usage.

The first version is a local web tool with:

- Rust backend API for health, runtime status, settings, and project registration stubs.
- React + TypeScript frontend for viewing status and editing configuration.
- Documentation placeholders for the future symbol graph, relationship index, and agent-facing query API.

## Why This Exists

Coding agents waste tokens when they repeatedly rediscover the same repository structure. Code Map is intended to become a compact navigation layer that answers:

- Which files should the agent read first?
- Which Controller / service / listener / mapper / config keys are likely relevant?
- What evidence backs a suggested code path?
- How much context should be returned for a fixed token budget?

## Project Layout

```text
code-map/
├── backend/              # Rust + Axum API server
├── frontend/             # React + TypeScript + Vite UI
├── docs/                 # Architecture, API, config, testing notes
├── AGENTS.md             # Agent maintenance rules
├── package.json          # Root command shortcuts
└── .env.example          # Local config template
```

## Quick Start

### Option A: systemd service (recommended)

The service runs the release backend, which also serves the built frontend.

```bash
cd /home/hevin/Developer/tools/code-map
npm run release          # build frontend dist + release backend
npm run deploy           # rebuild + restart the service
```

Open `http://127.0.0.1:18765`.

Service control:

```bash
systemctl --user status code-map
systemctl --user restart code-map
npm run logs             # journalctl --user -u code-map
```

The unit file lives at `deploy/code-map.service` and is installed to
`~/.config/systemd/user/code-map.service`. Linger is enabled, so the service
starts at boot.

### Option B: dev mode (hot reload)

```bash
cd /home/hevin/Developer/tools/code-map
npm --prefix frontend install   # first time only
npm run api                     # Rust backend on 127.0.0.1:18765
```

In another terminal:

```bash
cd /home/hevin/Developer/tools/code-map
npm run web                     # Vite dev server on 127.0.0.1:5178
```

Open the Vite URL shown by the frontend dev server.

## Common Commands

```bash
npm run api      # start Rust backend (dev)
npm run web      # start React frontend (dev, with API proxy)
npm run release  # build frontend dist + release backend binary
npm run deploy   # release build + restart systemd service
npm run build    # debug build of frontend + backend
npm run check    # frontend build + backend fmt check + backend tests
npm run status   # systemctl --user status code-map
npm run logs     # journalctl --user -u code-map
```

## Current Status

This is an initial scaffold. The UI can edit runtime settings and display backend state. Real code indexing and graph query features are intentionally not implemented yet.

## Next Milestones

1. Add persistent config storage under `CODE_MAP_DATA_DIR`.
2. Add repository scanner for files, languages, packages, and symbols.
3. Add relationship extraction for imports, routes, MQ listeners, jobs, and config keys.
4. Add `/api/query` endpoint that returns compact, evidence-backed navigation summaries.
5. Add agent CLI wrapper for `code-map query "business keyword"`.
