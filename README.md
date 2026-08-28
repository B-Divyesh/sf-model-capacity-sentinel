# Capacity Sentinel

Capacity Sentinel is a self-hosted canary runner for teams that depend on one or more model APIs. It sends synthetic prompts on a schedule, records vendor-neutral availability and elapsed-request latency, validates JSON response invariants, and opens an attributed alert after two consecutive failures.

It is not a production prompt proxy, model router, or semantic-quality judge. Production prompts never pass through this service.

## Included in v1

- OpenAI-compatible chat-completion probes against any HTTP(S) endpoint
- Manual and scheduled observations with 429, timeout, network, upstream, invalid-JSON, and invariant-failure classification
- Rolling availability and p95 latency evidence per provider/model
- Alerts after two consecutive failures and automatic recovery resolution
- AES-256-GCM API-key encryption with a unique local master key
- Daily per-probe token caps and response-token limits
- Local SQLite storage and unrestricted CSV export
- Keyboard/mobile-ready dashboard, offline/error/empty states, and privacy/terms pages
- Optional $39 Atlas license handoff through Sociobot; no payment provider is embedded

## Run locally

Prerequisites: Node 22+, npm, and Rust 1.90+.

```bash
npm ci
npm run build
DATA_DIR=./data cargo run
```

Open <http://localhost:8080>. The service creates `data/sentinel.db` and `data/master.key`; the key file is mode `0600` on Unix. Back up both together. To supply key material through a secret manager, set `SENTINEL_MASTER_KEY` (it is SHA-256-derived in memory and never logged).

For frontend development, run `npm run dev:server` and `npm run dev` in separate terminals. Vite proxies `/api` and `/health` to port 8080.

## Configure a canary

Provide the full OpenAI-compatible chat-completions URL, a provider label, model, API key, synthetic prompt, and optional comma-separated JSON dot paths such as `status, result.label`. The probe asks for a JSON object and checks that every declared path exists. It stores status, total request latency, token counts, and validation evidence—not response content.

The scheduler checks for due probes every 30 seconds. A probe interval can be 1–1,440 minutes. Two consecutive failures raise an availability alert; three or more samples above the p95 objective raise a latency alert. Recovered conditions resolve their open alert.

## Verify and build

```bash
npm test          # frontend unit + Rust unit/integration tests
npm run test:e2e # Playwright desktop + 390px mobile + axe checks
npm run check     # Svelte types + strict Clippy
npm run build     # production frontend -> dist/
```

## Container deployment

```bash
docker build --build-arg BUILD_SHA=$(git rev-parse --short HEAD) -t capacity-sentinel .
docker run --rm -p 8080:8080 -v sentinel-data:/data capacity-sentinel
```

The multi-stage image runs as a non-root distroless user, exposes port 8080, serves the built frontend, and persists SQLite plus encryption material under `/data`. `/health` returns status and the compile-time build SHA.

| Variable | Default | Purpose |
|---|---|---|
| `PORT` | `8080` | HTTP listen port |
| `DATA_DIR` | `./data` | SQLite and generated key directory |
| `STATIC_DIR` | `dist` | Built frontend directory |
| `SENTINEL_MASTER_KEY` | generated local key | External master-key material |
| `RUST_LOG` | environment default | Structured JSON log filter |

Place internet-exposed self-hosted instances behind your organization’s authentication gateway. Cross-origin browser calls are not enabled, request bodies are capped at 64 KB, and API credentials are never returned by the API.

## Load smoke

With the server running, test the read path at 100 requests/second using `oha`:

```bash
oha -z 10s -q 100 http://127.0.0.1:8080/health
```

## Data and billing

All operational data stays in the local SQLite volume. The only optional third-party call from the web app is an Atlas license verification request to `api.sociobot.in`. Sociobot/Dodo is merchant of record. See `/privacy` and `/terms` in the running application.

## License

MIT. See [LICENSE](LICENSE).
