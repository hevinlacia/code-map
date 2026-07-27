# Testing

## Current Automated Checks

```bash
npm --prefix frontend run build
cargo fmt --manifest-path backend/Cargo.toml -- --check
cargo test --manifest-path backend/Cargo.toml
```

Root shortcut:

```bash
npm run check
```

## Manual UI Smoke Test

1. Start service or dev mode:

   ```bash
   npm run deploy
   # or: npm run api + npm run web in separate terminals
   ```

2. Open `http://127.0.0.1:18765` for systemd mode or the Vite URL for dev mode.
3. Confirm status cards load.
4. Add a repository, for example `/home/hevin/Developer/tools/code-map`.
5. Click **Scan** and confirm file count/bytes update.
6. Query `settings`, `projects`, `/api/query`, or `RuntimeState`.
7. Confirm the result list shows ranked files and line snippets.

## API Smoke Test

```bash
BASE=http://127.0.0.1:18765
curl "$BASE/health"
curl "$BASE/api/status"
curl "$BASE/api/settings"
```

Create, scan, and query this project:

```bash
BASE=http://127.0.0.1:18765
ROOT=/home/hevin/Developer/tools/code-map
PROJECT_ID=$(curl -fsS -X POST "$BASE/api/projects" \
  -H 'Content-Type: application/json' \
  -d "{\"name\":\"code-map\",\"root_path\":\"$ROOT\"}" | jq -r '.id')

curl -fsS -X POST "$BASE/api/projects/$PROJECT_ID/scan" | jq '{name,indexed,file_count,total_bytes}'

curl -fsS -X POST "$BASE/api/query" \
  -H 'Content-Type: application/json' \
  -d "{\"project_id\":\"$PROJECT_ID\",\"query\":\"settings\",\"max_results\":5}" \
  | jq '{query,result_count,first_result: .results[0]}'
```

If the project already exists, use `GET /api/projects` to reuse its ID.

## CLI Smoke Test

```bash
./scripts/code-map-query.sh settings
npm run query -- settings
```
