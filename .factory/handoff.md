# Capacity Sentinel repair handoff — PASS

Repair work order `model-capacity-sentinel-repair-2`, based on independent
verification report commit `4c565ac7a3c483be3e3ec617be0a24328c309f21`
for candidate `1a31ab744b4f76184b70f6f8ee36b16b0dfc403b`.

## Release findings repaired

- **Immutable live identity:** `BUILD_SHA` is now one global Docker build
  argument, inherited by the Rust builder and final image. Rust compiles the
  value into `/health`; the runtime image no longer replaces it with
  `unknown`. The final image also carries
  `org.opencontainers.image.revision`. A contract test prevents the broken
  two-default Dockerfile layout from returning.
- **Clean unauthenticated first paint:** the browser now renders the access
  recovery state without requesting `/api/summary`. It starts loading only
  after a session access code exists and likewise suppresses reconnect and
  polling requests while locked. A credential-free desktop/mobile Playwright
  regression asserts zero summary requests and zero console errors before
  unlock, then verifies keyboard Enter recovery.
- **Visible, non-secret startup provenance:** logging defaults to `info` when
  `RUST_LOG` is absent. Startup emits one structured
  `startup_configuration` record with default/supplied path and port sources,
  generated/persisted/supplied master-key and access-token sources, and the
  build identity. Values of secrets and paths are not logged. A real-process
  Rust test launches without `RUST_LOG` and verifies the generated-source
  record and mode-0600 secret files.

The researched brief, field-guide visual system, encrypted-canary behavior,
SSRF controls, authenticated API, paid unlock, and artifact/deployment class
remain unchanged.

## Verification evidence

Run from a clean dependency installation on 2026-08-28:

```text
npm ci                                      PASS; 134 packages, 0 vulnerabilities
npm test                                    PASS; 3 Vitest + 7 Rust + 1 process integration
npm run check                               PASS; Svelte 0 errors/warnings; Clippy -D warnings
npm run build                               PASS; dist/ produced
npm run test:e2e                            PASS; 6/6 desktop + 390×844 mobile
npm audit --omit=dev                        PASS; 0 vulnerabilities
BUILD_SHA=repair-verification cargo build --locked --release
                                             PASS
```

Production assets remain inside budget: JavaScript 67,533 bytes raw / 25,075
bytes gzip; CSS 16,991 bytes raw / 4,644 bytes gzip; hero WebP 129,198 bytes.
The product remains well below the 200 KB initial-JS, 50 KB CSS, and 300 KB
mobile-hero ceilings.

The release binary was also exercised over HTTP:

- no-environment startup used port 8080, generated both secrets as mode 0600,
  logged all configuration sources at the default level, and returned
  `{"build":"repair-verification","status":"ok"}`;
- unauthenticated summary returned 401; authenticated create/summary passed;
  the API response omitted the canary and SQLite/WAL string inspection found
  neither the supplied API key nor canary;
- a loopback endpoint returned 400, a 70 KiB write returned 413, and 100/100
  concurrent health requests returned 200;
- HTML/service worker returned `no-cache`, hashed JS returned one-year
  `immutable`, API/health returned `no-store`, and CSP, frame, content-type,
  and referrer headers were present.

Independent browser smoke against that release binary covered 1440×900 and
390×844. Both had one `h1`, one `main`, no horizontal overflow, no console or
page errors, no third-party request origins, and zero serious/critical axe
findings. Dialog focus and Escape, access-code Enter recovery, legal routes,
populated and empty states, reduced motion, and keyboard focus remain covered.
The active service worker completed an update check and `/privacy` reloaded
offline. `/opt/fleet/lib/verify-url.sh` passed with title, `lang=en`, landmark,
image-alt and control-name checks. Mobile Lighthouse: performance 98,
accessibility 100, best practices 100, SEO 100; LCP 1.9 s, CLS 0, total
blocking time 130 ms.

## Deployment

The factory container deployment uses:

```bash
/opt/fleet/lib/deploy-container.sh model-capacity-sentinel /work/repo Dockerfile 8080
```

That workflow builds in ACR with `BUILD_SHA`, `GIT_SHA`, and `SOURCE_COMMIT`
all set to `git rev-parse HEAD`, then updates the existing Azure Container App.
Post-deploy verification checks HTTPS, browser console/accessibility basics,
response policy, and requires live `/health.build` to equal the deployed
40-character commit.

## Known gaps

- Docker is unavailable inside this worker, so there is no local Docker-engine
  build result. The required multi-stage image is built by the configured ACR
  deployment, while native locked release compilation and the running release
  server provide pre-deploy coverage.
- This remains the intended single-project self-hosted runner. A managed
  multi-tenant edition would require account-level project isolation and is
  outside this repair.
