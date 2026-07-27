# Architecture

Code Map is designed as an agent-facing repository navigation and context compression layer.

## Goals

- Reduce repeated blind repository exploration by coding agents.
- Return compact summaries that point to concrete files, symbols, and evidence lines.
- Keep UI focused on runtime state, configuration, index health, and project management.
- Keep source code extraction deterministic and auditable before adding LLM summaries.

## Non-Goals For The Scaffold

- No full graph database yet.
- No LLM summarization yet.
- No destructive filesystem operations from the UI.
- No production deployment assumptions.

## Components

### Frontend

The frontend is a Vite React app in `frontend/`.

Responsibilities:

- Display backend health and runtime status.
- Edit safe runtime settings.
- Show registered/indexed workspaces.
- Later: show index freshness, symbol counts, and query traces.

### Backend

The backend is an Axum service in `backend/`.

Responsibilities:

- Expose state and settings APIs.
- Own the future project index and query API.
- Enforce token/summary budgets for agent-facing responses.
- Keep route handlers thin and move feature logic under `backend/src/features/`.

### Future Index Pipeline

Planned stages:

1. Repository discovery: files, languages, package manifests, module boundaries.
2. Symbol extraction: classes, functions, routes, listeners, jobs, mappers, config keys.
3. Relationship extraction: imports, calls, API routes, MQ producers/consumers, DB tables.
4. Query planning: rank candidate files and return next-read recommendations.
5. Evidence compaction: emit source-backed summaries with paths and line ranges.

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
