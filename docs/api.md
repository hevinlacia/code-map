# API Contract

Base URL during local development and systemd mode: `http://127.0.0.1:18765`.

The Vite dev server proxies `/api` and `/health` to the backend. In systemd mode,
the backend serves both API and the built frontend.

## Error Shape

```json
{
  "code": "bad_request",
  "message": "human readable message"
}
```

## `GET /health`

```json
{
  "ok": true,
  "service": "code-map-backend"
}
```

## `GET /api/settings`

Returns persisted runtime settings from `CODE_MAP_DATA_DIR/settings.json`.

```json
{
  "indexing_enabled": true,
  "auto_refresh_enabled": false,
  "active_project_id": "optional-project-uuid",
  "query_token_budget": 2000,
  "max_summary_lines": 50
}
```

## `PUT /api/settings`

Updates and persists runtime settings. `auto_refresh_enabled` is forced to `false`
unless `CODE_MAP_ENABLE_WRITE_ACTIONS=true`.

## `GET /api/projects`

Returns persisted project summaries from `CODE_MAP_DATA_DIR/projects.json`.

```json
[
  {
    "id": "generated-uuid",
    "name": "WMS workspace",
    "root_path": "/home/hevin/Developer/company/WMS",
    "indexed": true,
    "last_indexed_at": "1785140000",
    "file_count": 26676,
    "repo_count": 32,
    "total_bytes": 127051610,
    "symbol_count": 49309,
    "relationship_count": 8218
  }
]
```

## `POST /api/projects`

Registers a local repository directory or multi-repo workspace.

```json
{
  "name": "WMS workspace",
  "root_path": "/home/hevin/Developer/company/WMS"
}
```

## `POST /api/projects/{id}/scan`

Scans a registered repository or workspace into a lightweight index.

Query parameter `force=true` forces a full re-read and rebuilds the constant
table (use after constant definitions change). Default is incremental: only
changed files (by mtime+size) are re-read and re-extracted; unchanged files reuse
cached symbols/relationships, and the constant table is loaded from
`CODE_MAP_DATA_DIR/constants.json`.

Current scanner behavior:

- Detects nested Git repositories and preserves repo boundaries.
- Scans each Git repository separately when a workspace contains multiple repos.
- Skips WMS worktree/generated noise such as `.worktrees`, `.umi`, `.git`, `node_modules`, `target`, `dist`, `.serena`, `.codegraph`, and `session-*.md`.
- Extracts heuristic symbols: classes/interfaces/enums, Controller routes, MQ topics/tags/groups, MQ components, Feign/Dubbo references, Mapper hints, DB tables, frontend API calls, and frontend permission strings.
- Extracts heuristic relationships: MQ publish/consume, Feign/Dubbo references, SQL table read/write, and frontend API calls.

## `POST /api/query`

Searches an indexed project for keywords, endpoint fragments, class names, config
keys, table names, repo names, MQ topic strings, symbols, and relationship
endpoints.

```json
{
  "project_id": "project-uuid",
  "query": "WMS_SHIPMENT_UPLOAD_TOPIC",
  "max_results": 5
}
```

`project_id` may be `null` if there is exactly one project or an active project
is set in settings.

```json
{
  "project_id": "project-uuid",
  "project_name": "WMS workspace",
  "query": "WMS_SHIPMENT_UPLOAD_TOPIC",
  "terms": ["wms_shipment_upload_topic", "wms-shipment-upload-topic", "wmsshipmentuploadtopic"],
  "result_count": 5,
  "summary_lines": [
    "Project 'WMS workspace' query 'WMS_SHIPMENT_UPLOAD_TOPIC' returned 5 candidate file(s) across 32 repo(s).",
    "1. backend/yl-cwhsea-wms-api:wms-common/src/main/java/com/ztocwst/wms/common/rocketmq/constant/ShipmentTopicConstants.java:61 score=54 reason=symbol detail contains 'wms_shipment_upload_topic'",
    "Suggested agent action: read the top 3 repo/path line hints first."
  ],
  "results": [
    {
      "relative_path": "backend/yl-cwhsea-wms-api/wms-common/src/main/java/com/ztocwst/wms/common/rocketmq/constant/ShipmentTopicConstants.java",
      "repo": "backend/yl-cwhsea-wms-api",
      "repo_relative_path": "wms-common/src/main/java/com/ztocwst/wms/common/rocketmq/constant/ShipmentTopicConstants.java",
      "language": "Java",
      "score": 54,
      "reasons": ["symbol detail contains 'wms_shipment_upload_topic'"],
      "symbols": [
        {
          "kind": "mq_topic",
          "name": "wms-shipment-upload-topic",
          "detail": "public static final String WMS_SHIPMENT_UPLOAD_TOPIC = \"wms-shipment-upload-topic\";",
          "relative_path": "backend/yl-cwhsea-wms-api/wms-common/src/main/java/com/ztocwst/wms/common/rocketmq/constant/ShipmentTopicConstants.java",
          "repo": "backend/yl-cwhsea-wms-api",
          "repo_relative_path": "wms-common/src/main/java/com/ztocwst/wms/common/rocketmq/constant/ShipmentTopicConstants.java",
          "line": 61
        }
      ],
      "relationships": [],
      "snippets": [
        { "line": 61, "text": "public static final String WMS_SHIPMENT_UPLOAD_TOPIC = \"wms-shipment-upload-topic\";" }
      ]
    }
  ]
}
```

Common symbol kinds:

- `class`, `interface`, `enum`
- `controller`, `controller_route`
- `mq_topic`, `mq_tag`, `mq_group`, `mq_component`, `mq_consumer`
- `feign_client`, `dubbo_reference`, `dubbo_service`
- `mapper`, `db_table`
- `frontend_api_call`, `frontend_permission`

Common relationship kinds:

- `mq_publish`, `mq_consume`
- `feign_client`, `dubbo_reference`
- `sql_table_read`, `sql_table_write`
- `frontend_calls_api`

## `POST /api/notes` and `GET /api/notes`

Verified notes cache agent-confirmed call chains and conclusions. When a query
fuzzy-matches an existing note, the note is surfaced inline in the query
response `notes` field, so the next agent does not re-explore the same chain.

Create a note:

```json
{
  "project_id": "project-uuid",
  "query": "ShipmentUploadMq",
  "summary": "human-readable confirmed conclusion",
  "pointers": [{"repo": "...", "path": "...", "line": 61, "note": "main class"}]
}
```

List notes (fuzzy match by query):

```json
{ "project_id": "project-uuid", "query": "shipment" }
```

Notes persist in `CODE_MAP_DATA_DIR/notes.json`.

## CLI Helper

```bash
# ranked candidate files (agent: add --json for compact output)
./scripts/code-map-query.sh query ShipmentUploadMq --json --max-results 8

# resolve entity to producers/consumers/readers/writers/callers
./scripts/code-map-query.sh neighbors wms-shipment-upload-topic --json
./scripts/code-map-query.sh neighbors shipment_header --json

# backward compat: positional arg = query
./scripts/code-map-query.sh ShipmentUploadMq
```

Optional environment variables:

- `CODE_MAP_BASE_URL` - default `http://127.0.0.1:18765`.
- `CODE_MAP_PROJECT_ID` - query a specific project.
- `CODE_MAP_MAX_RESULTS` - default `12`.

## `POST /api/neighbors`

Resolves an entity (topic string, table name, Feign client, class name, or
constant) to its neighbors across all indexed repos. Designed for impact
questions: "who consumes this topic", "who writes this table", "who calls this
Feign client" -- answered in one call without reading source.

```json
{
  "project_id": "project-uuid",
  "entity": "shipment_header"
}
```

Response (each hit is `repo:path:line` + kind + name, capped at 25 per bucket):

```json
{
  "project_id": "project-uuid",
  "project_name": "WMS workspace",
  "entity": "shipment_header",
  "definitions": [{"kind":"db_table","name":"shipment_header","repo":"...","repo_relative_path":"...","line":15}],
  "producers": [],
  "consumers": [],
  "readers": [{"kind":"sql_table_read","name":"backend/yl-cwhsea-wms-api:...","repo":"...","repo_relative_path":"...","line":39}],
  "writers": [{"kind":"sql_table_write","name":"...","repo":"...","repo_relative_path":"...","line":51}],
  "callers": []
}
```

Bucket semantics:

- `definitions` - symbols whose name matches the entity.
- `producers` - `mq_publish` relationships targeting the entity.
- `consumers` - `mq_consume` relationships targeting the entity.
- `readers` - `sql_table_read` relationships targeting the entity.
- `writers` - `sql_table_write` relationships targeting the entity.
- `callers` - `feign_client` / `dubbo_reference` / `frontend_calls_api` targeting the entity.

Note: entities referenced via constants (e.g. `ShipmentTopicConstants.WMS_SHIPMENT_UPLOAD_TOPIC`)
instead of string literals may not resolve into producers/consumers; check
`definitions` to find the constant file first.
