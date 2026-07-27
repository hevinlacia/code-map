# Code Map Roadmap

## Phase 0 - Scaffold ✅

- Rust backend with health, status, settings, and project APIs.
- React frontend for status and safe runtime configuration.
- Project docs, local command shortcuts, and systemd service.

## Phase 1 - Working MVP ✅

- Persist runtime settings and registered projects under `CODE_MAP_DATA_DIR`.
- Add project registration UI.
- Scan repository files with generated/cache directory ignore rules.
- Record lightweight file index: relative path, language guess, size, line count, and text-search eligibility.
- Add `POST /api/query` keyword search over indexed paths and text lines.
- Add CLI helper: `./scripts/code-map-query.sh <query>`.

## Phase 2 - WMS Heuristic Symbols And Relationships ✅

- Detect nested Git repositories and preserve repo boundaries.
- Extract heuristic symbols: classes/interfaces/enums, Controller routes, MQ topics/tags/groups, MQ components, Feign/Dubbo references, Mapper hints, DB tables, frontend API calls, and frontend permission strings.
- Extract heuristic relationships: MQ publish/consume, Feign/Dubbo references, SQL table read/write, and frontend API calls.
- Rank query results with symbol and relationship hits in addition to path/content matches.
- Filter WMS noise such as `.worktrees`, frontend `.umi`, generated/cache folders, and `session-*.md` files.

## Phase 3 - Better Index Quality

- Add file hash and modified time for freshness checks.
- Add package/module discovery (`Cargo.toml`, `package.json`, `pom.xml`, `build.gradle`, etc.).
- Add re-scan status, progress, and errors.
- Add delete/edit project controls.
- Add JSON output mode to the CLI helper.

## Phase 4 - Parser-Backed Graph

- Replace heuristic extraction with parser-backed extraction via tree-sitter or language-specific parsers.
- Resolve relationships across repos (e.g. topic producer → consumer, frontend API → backend controller, mapper → table).
- Add graph traversal queries such as entrypoint → service → MQ/DB/downstream repo.

## Phase 5 - Agent Memory

- Cache verified exploration notes for reuse.
- Store confirmed call chains and source-backed summaries.
- Add query history and “verified by agent” flags.
- Add compact machine-readable responses optimized for coding agents.
