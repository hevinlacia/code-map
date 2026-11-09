# Deployment

Code Map runs as a user-level systemd service, matching the convention used by
`agent-panel`, `api-workbench`, and `sql-workbench`.

## Architecture

A single service runs the release backend binary. The backend serves:

- `/api/*` and `/health` — JSON API.
- `/*` — built frontend static assets from `frontend/dist` (via `tower-http`
  `ServeDir`), with SPA fallback to `index.html`.

There is no separate frontend process in production. Dev mode still uses the
Vite dev server with an API proxy.

## Unit File

The canonical unit file is version-controlled at `deploy/code-map.service` and
installed to `~/.config/systemd/user/code-map.service`.

Key settings:

- `WorkingDirectory` — project root.
- `CODE_MAP_HOST` / `CODE_MAP_PORT` — bind `127.0.0.1:18765`.
- `CODE_MAP_DEFAULT_WORKSPACE` — optional default workspace, currently `/home/hevin/Developer/company/WMS` in the installed unit.
- `CODE_MAP_DATA_DIR` — `~/.local/share/code-map` for persisted settings, projects, and file indexes.
- `CODE_MAP_FRONTEND_DIR` — absolute path to `frontend/dist`.
- `CODE_MAP_ENABLE_WRITE_ACTIONS=false` — write toggles disabled until explicitly enabled.
- `Restart=on-failure`, `RestartSec=2`, `NoNewPrivileges=true`.
- `WantedBy=default.target`; linger is enabled so the service starts at boot.

## Build And Deploy

```bash
cd /home/hevin/Developer/tools/code-map
npm run release   # build frontend dist + release backend binary
npm run deploy    # release build + systemctl --user restart code-map
```

After changing the unit file, reinstall and reload:

```bash
install -m644 deploy/code-map.service ~/.config/systemd/user/code-map.service
systemctl --user daemon-reload
systemctl --user restart code-map
```

## Service Control

```bash
systemctl --user status code-map
systemctl --user restart code-map
systemctl --user stop code-map
npm run logs     # journalctl --user -u code-map -n 200 --no-pager
journalctl --user -u code-map -f     # live tail
```

## Verification

```bash
curl http://127.0.0.1:18765/health
curl http://127.0.0.1:18765/api/status
curl -I http://127.0.0.1:18765/      # should return index.html
```

## Notes

- Runtime settings and project indexes are persisted under `CODE_MAP_DATA_DIR`.
- Logs go to journald (no separate log file), consistent with `sql-workbench`.

## Hub artifact distribution (REQ-274)

New nodes can run code-map **without cloning this repo**: hub holds the
packaged artifact and the panel worker installs it on demand.

```bash
npm run release              # build frontend dist + release backend
deploy/package.sh            # → dist/code-map-<git-short>.tar.gz (+ upload command)
```

Package layout:

```
code-map-<ver>/
  bin/code-map-backend    # platform-bound binary (x86_64-unknown-linux-gnu)
  frontend/               # Vite build output (dist)
  install.env.example     # runtime env template
```

Upload with `kind=binary` (entry goes in the **query string**, URL-encoded —
`--data-urlencode` would corrupt the tar.gz body). Then declare `artifact` in the
node's service manifest; the worker downloads, verifies sha256, unpacks to
`~/.local/share/panel-services/code-map/<ver>/` and reports `artifactVersion`
via `GET /api/services`. Contract details live in the agent-panel skill:
`.agents/skills/agent-panel/references/workers-api.md` → 服务产物分发.

**Platform note:** the binary only runs on the same architecture/libc as the
build machine. Cross-distro or ARM nodes need their own build uploaded as a
separate version. Point `CODE_MAP_FRONTEND_DIR` at the unpacked `frontend/`
inside the install dir.
