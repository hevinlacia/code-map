# Code Map Initial Roadmap

## Phase 0 - Scaffold

- Rust backend with health, status, settings, and project placeholder APIs.
- React frontend for status and safe runtime configuration.
- Project docs and local command shortcuts.

## Phase 1 - Persistent Settings

- Persist runtime settings and registered projects under `CODE_MAP_DATA_DIR`.
- Add schema versioning for local config files.
- Add UI project add/edit/delete flows.

## Phase 2 - Repository Discovery

- Scan workspace files with ignore rules.
- Detect language, package manager, manifests, and module boundaries.
- Record file hash, modified time, and index freshness.

## Phase 3 - Symbol And Relationship Index

- Extract symbols via tree-sitter or language-specific parsers.
- Index HTTP routes, MQ listeners, scheduled jobs, config keys, imports, and selected DB references.
- Add evidence pointers with file path and line ranges.

## Phase 4 - Agent Query API

- Add `POST /api/query`.
- Rank candidate files for business keywords, endpoint paths, MQ topics, table names, and config keys.
- Return compact summaries constrained by `query_token_budget` and `max_summary_lines`.

## Phase 5 - CLI Integration

- Add `code-map query "keyword"` wrapper.
- Support JSON output for agents and readable output for humans.
- Cache verified exploration notes for reuse.
