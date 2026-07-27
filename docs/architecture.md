# Architecture

Code Map is designed as an agent-facing repository navigation and context compression layer.

## Goals

- Reduce repeated blind repository exploration by coding agents.
- Return compact summaries that point to concrete files, symbols, and evidence lines.
- Keep UI focused on runtime state, configuration, index health, and project management.
- Keep source code extraction deterministic and auditable before adding LLM summaries.

## Current Non-Goals

- No full graph database yet.
- No LLM summarization yet.
- No destructive filesystem operations from the UI.
- No real symbol/relationship extraction yet; the current MVP is file/path/line search.

## Components

### Frontend

The frontend is a Vite React app in `frontend/`.

Responsibilities:

- Display backend health and runtime status.
- Edit safe runtime settings.
- Add registered repository paths.
- Trigger repository scans.
- Query indexed projects and display candidate files, reasons, scores, and line snippets.
- Later: show index freshness, symbol counts, relationship traces, and query history.

### Backend

The backend is an Axum service in `backend/`.

Responsibilities:

- Expose state, settings, project, scan, and query APIs.
- Persist settings and project indexes under `CODE_MAP_DATA_DIR`.
- Enforce safe settings validation and indexing gates.
- Return compact, evidence-backed query summaries for agent workflows.
- Keep route handlers thin and move feature logic under `backend/src/features/`.

### Current Index Pipeline

Implemented stages:

1. Project registration: canonical local repository path persisted to JSON.
2. Repository discovery: detect nested Git repositories in a workspace and scan each repo separately.
3. WMS heuristic extraction: classes/interfaces/enums, Controller routes, MQ topics/tags/groups, MQ components, Feign/Dubbo references, Mapper hints, DB tables, frontend API calls, and frontend permission strings.
4. Relationship extraction: MQ publish/consume, Feign/Dubbo references, SQL table read/write, and frontend API calls.
5. Keyword query: rank files by repo name, repo-relative path, workspace path, text-line matches, symbol matches, and relationship matches.
6. Evidence compaction: emit summary lines with `repo:path`, line hints, scores, reasons, symbols, relationships, and snippets.

### Future Index Pipeline

Planned stages:

1. Replace heuristic extractors with parser-backed extraction via tree-sitter or language-specific parsers.
2. Resolve relationship endpoints across repositories instead of storing only string targets.
3. Query planning: rank by graph proximity and verified business knowledge, not just local match score.
4. Verified notes: store confirmed call chains and agent exploration outcomes.
5. JSON CLI mode: direct machine-readable output for coding agents.

## Data Flow

```text
React UI -> Backend API -> Runtime state / future index store
Agent CLI -> Backend query API -> Compact evidence-backed summary
Repository files -> Scanner -> Index store -> Query API
```

## Safety Model

- Runtime write actions are disabled unless `CODE_MAP_ENABLE_WRITE_ACTIONS=true`.
- Generated index data belongs under `CODE_MAP_DATA_DIR` and is gitignored.
- Agent-facing results should cite file paths and line ranges instead of copying large source blocks.
