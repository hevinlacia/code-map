# AGENTS.md - code-map

## Scope

This project is an agent-oriented code map tool under `/home/hevin/Developer/tools/code-map`.

- Backend: Rust + Axum + Tokio in `backend/`
- Frontend: TypeScript + React + Vite in `frontend/`
- Primary goal: return compact, evidence-backed repository navigation summaries for coding agents.

## Architecture Entry

- Frontend entry: `frontend/src/main.tsx`
- Frontend API client: `frontend/src/api/client.ts`
- Backend entry: `backend/src/main.rs`
- Backend router: `backend/src/app.rs` and `backend/src/routes/`
- API docs: `docs/api.md`
- Architecture docs: `docs/architecture.md`
- Deployment docs: `docs/deployment.md`
- Config docs: `docs/config.md`

## Commands

- Install frontend dependencies: `npm --prefix frontend install`
- API dev server: `npm run api`
- Web dev server: `npm run web`
- Release build (frontend dist + release backend): `npm run release`
- Deploy (release build + restart service): `npm run deploy`
- Service status: `npm run status` or `systemctl --user status code-map`
- Service logs: `npm run logs` or `journalctl --user -u code-map -f`
- Build all (debug): `npm run build`
- Check all: `npm run check`
- Backend format: `cargo fmt --manifest-path backend/Cargo.toml`
- Backend tests: `cargo test --manifest-path backend/Cargo.toml`

## Rules

- Keep API contracts documented in `docs/api.md` whenever endpoints or DTOs change.
- Keep frontend API types in `frontend/src/api/client.ts` aligned with backend response structs.
- Backend route handlers stay thin; real indexing/query logic belongs under `backend/src/features/`.
- Do not hardcode personal workspace paths in source code; use env vars and document defaults.
- Do not commit `.env`, generated data, build outputs, or local index cache files.
- Prefer compact, evidence-backed summaries over dumping source code into responses.
