# Capacity Sentinel

Capacity Sentinel monitors model API capacity, latency, and JSON response shape
with synthetic canary requests. It is for teams that operate applications using
one or more model APIs.

Start with [the one-click sample](/demo). It shows two model endpoints, a 429
capacity alert, and a recovered alert without opening a project.

## What it does

- Keeps one project on its own self-hosted runner.
- Runs synthetic OpenAI-compatible chat-completion probes on a schedule.
- Records status, elapsed request time, and required JSON-field checks.
- Opens provider/model-specific alerts after repeated failures.
- Lets operators edit probes, delete their local history, and export CSV.
- Keeps credentials and synthetic prompts encrypted in the local runner.

It does not proxy production prompts, route production traffic, or judge model
truthfulness or general quality.

## Demo

Open `/demo` or choose **Try it with sample data** on the landing page. Demo
data lives only under the browser-local `demo:capacity-sentinel:summary` key.
It never calls the project API or writes runner data. **Reset demo** restores
the original sample; **Start for real** opens the separate access-code screen.
See [`.factory/demo.md`](.factory/demo.md) for the exact sample and isolation
boundary.

## Run locally

Prerequisites: Node 22+, npm, and current stable Rust.

```bash
npm ci
npm run build
PORT=8080 cargo run
```

Open <http://localhost:8080>. With no secret configuration, the runner creates
an access code and encryption material in `./data`. In a container it uses the
durable `/data` mount when present. Set `SENTINEL_ACCESS_TOKEN` to provide a
project access code; set `SENTINEL_MASTER_KEY` only when you manage the
encryption key outside the runner.

Use the project access code for every `/api/*` request. API requests are
limited per client before access-code validation. Excess requests receive 429
and `Retry-After`.

## Verify from a clean checkout

```bash
npm ci
npm test
npm run check
npm run build
npm run test:e2e
npm run test:claims
BUILD_SHA=local-check cargo build --locked --release
```

`npm run test:claims` runs every command in
[`.factory/claims.json`](.factory/claims.json). Each browser claim starts from
the shipped `/demo` entry point or another explicitly described temporary
runner sandbox.

## Container deployment

```bash
docker build --build-arg BUILD_SHA=$(git rev-parse HEAD) -t capacity-sentinel .
docker run --rm -p 8080:8080 -v sentinel-data:/data capacity-sentinel
```

The multi-stage image runs as non-root, listens on `PORT` (default 8080), and
needs no required environment variables. It stores SQLite, generated keys, and
the generated access code under `/data`. `/health` returns its baked build
identity.

| Variable | Default | Purpose |
|---|---|---|
| `PORT` | `8080` | HTTP listen port |
| `DATA_DIR` | `/data` when mounted, otherwise `./data` | SQLite and generated key directory |
| `STATIC_DIR` | `/app/dist` in container, otherwise `dist` | Built frontend directory |
| `SENTINEL_MASTER_KEY` | generated local key | External master-key material |
| `SENTINEL_ACCESS_TOKEN` | generated local token | Project access code |

## Atlas add-on

Atlas is a $39 one-time paid add-on for the 365-day comparison view. The free
runner keeps probe editing, alerts, safety information, and CSV export. The
Sociobot billing registration for this product is currently pending, so checkout
is not offered yet. License restore and verification use Sociobot when a valid
license is available.

## Privacy and license

Read `/privacy` and `/terms` in the running product. Capacity Sentinel has no
analytics or third-party scripts. The optional license verification request is
made to Sociobot only after a visitor chooses to restore or verify a license.

MIT. See [LICENSE](LICENSE).
