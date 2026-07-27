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

## Manual Smoke Test

1. Start backend:

   ```bash
   npm run api
   ```

2. Start frontend in another terminal:

   ```bash
   npm run web
   ```

3. Open the Vite URL, usually `http://127.0.0.1:5178`.
4. Confirm status cards load.
5. Toggle settings and click **Save settings**.
6. Confirm settings reload without an error alert.

## API Smoke Test

```bash
curl http://127.0.0.1:18765/health
curl http://127.0.0.1:18765/api/status
curl http://127.0.0.1:18765/api/settings
```
