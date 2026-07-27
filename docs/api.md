# API Contract

Base URL during local development: `http://127.0.0.1:18765`.

The Vite frontend proxies `/api` and `/health` to the backend.

## Error Shape

```json
{
  "code": "bad_request",
  "message": "human readable message"
}
```

## `GET /health`

Returns process health.

Response:

```json
{
  "ok": true,
  "service": "code-map-backend"
}
```

## `GET /api/status`

Returns current runtime status for the UI.

Response:

```json
{
  "service": "code-map",
  "indexing_enabled": true,
  "auto_refresh_enabled": false,
  "active_project_id": null,
  "query_token_budget": 2000,
  "max_summary_lines": 50
}
```

## `GET /api/settings`

Returns editable runtime settings.

Response:

```json
{
  "indexing_enabled": true,
  "auto_refresh_enabled": false,
  "active_project_id": null,
  "query_token_budget": 2000,
  "max_summary_lines": 50
}
```

## `PUT /api/settings`

Updates runtime settings.

Request:

```json
{
  "indexing_enabled": true,
  "auto_refresh_enabled": false,
  "active_project_id": null,
  "query_token_budget": 2000,
  "max_summary_lines": 50
}
```

Validation:

- `query_token_budget` must be at least `200`.
- `max_summary_lines` must be between `1` and `500`.
- `auto_refresh_enabled` is forced to `false` unless `CODE_MAP_ENABLE_WRITE_ACTIONS=true`.

## `GET /api/projects`

Returns registered project summaries. Current implementation returns a default workspace placeholder.

Response:

```json
[
  {
    "id": "generated-uuid",
    "name": "Default workspace",
    "root_path": "Configure CODE_MAP_DEFAULT_WORKSPACE",
    "indexed": false,
    "last_indexed_at": null,
    "symbol_count": 0,
    "relationship_count": 0
  }
]
```

## `POST /api/projects`

Creates a project summary placeholder. Persistence is not implemented yet.

Request:

```json
{
  "name": "WMS backend",
  "root_path": "/path/to/repository"
}
```

Response status: `201 Created`.

## Planned Agent Query API

Future endpoint:

```text
POST /api/query
```

Expected responsibility:

- Accept a keyword, endpoint path, MQ topic, table name, config key, or file path.
- Return ranked candidate files and relationships.
- Keep response under `query_token_budget`.
- Include evidence pointers using `repo:path:start_line-end_line`.
