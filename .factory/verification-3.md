# Independent verification 3 — FAIL

Verified 2026-08-28 against commit `1a31ab744b4f76184b70f6f8ee36b16b0dfc403b` on `main` and <https://model-capacity-sentinel.sociobot.in>.

## Decision

**FAIL.** The candidate is locally buildable and its deployed frontend exactly matches the candidate, but the live backend does not expose an immutable build identity. The deployment therefore cannot be confirmed as this candidate, as required by the backend contract and this verification work order.

## Release-blocking defect

### High — live backend build identity is `unknown`

Fresh `GET /health` from the deployed URL returned:

```json
{"build":"unknown","status":"ok"}
```

The expected value is `1a31ab744b4f76184b70f6f8ee36b16b0dfc403b`. The candidate source supports `BUILD_SHA`, and a locally started release binary returned that exact value when supplied, so this is a deployment/build-identity failure. Without it, the live Rust backend cannot be tied to the accepted source even though its static frontend is proven to match.

Required remediation: rebuild/redeploy with `--build-arg BUILD_SHA=1a31ab744b4f76184b70f6f8ee36b16b0dfc403b` (or equivalent immutable factory value), then confirm `/health` exposes it.

## Other defects

### Medium — first normal deployed visit produces a browser console error

On both desktop and 390px mobile, an unauthenticated first visit loads the access-code recovery screen correctly but first requests `/api/summary`. The expected HTTP 401 is emitted by Chromium as `Failed to load resource: the server responded with a status of 401 ()`. This fails the no-console-errors-on-load quality gate, which repository tests miss by pre-seeding the access code.

Required remediation: avoid the summary request until an access code is available, or make the unauthenticated bootstrap response not generate a console error while preserving the explicit recovery UI.

### Medium — required generated-versus-supplied config startup record is absent

The no-environment runtime started successfully on default port 8080 and generated `data/master.key` and `data/access.token` (both mode 0600). With `RUST_LOG=info`, the only startup record was `capacity sentinel listening`; it does not state which secret/config was generated versus supplied, contrary to the backend runtime configuration contract. With no `RUST_LOG`, there was no startup record at all because the default filter suppresses info output.

Required remediation: emit a non-secret startup configuration-source record at the effective default log level.

## Passing evidence

- Clean checkout was exactly `1a31ab744b4f76184b70f6f8ee36b16b0dfc403b`. `npm ci` completed with 0 vulnerabilities.
- `npm test` passed: 2 Vitest tests and 6 Rust tests. `npm run check` passed: Svelte check 0 errors/0 warnings and `cargo clippy -- -D warnings` clean. `npm run build` passed. `npm run test:e2e` passed 4/4 desktop/mobile tests. `cargo build --locked --release` completed successfully.
- Built assets are within budget: JS 67,499 B raw / 25,440 B gzip; CSS 16,991 B raw / 4,660 B gzip; hero WebP 129,198 B. No external fonts/scripts are loaded.
- Independent release-server API exercise: unauthenticated summary 401; loopback probe creation 400; out-of-range schedule/objective 400; a valid public `https://httpbin.org/status/429` canary produced two 429 `capacity` observations and an attributed availability alert after the second run. A 70 KiB write was 413. One hundred concurrent `/health` requests all returned 200.
- The API does not return prompts. Local database string inspection did not contain the supplied QA API key or synthetic prompt; `master.key` was 0600. A generated access token and SQLite database survived a no-environment server restart.
- Local browser checks on a populated dashboard at 1440px and 390px found one `h1`, one `main`, no horizontal overflow, no console/page errors in normal authenticated use, and zero axe serious/critical findings. Keyboard focus was visibly `rgb(23, 95, 132) solid 3px`; dialog first focus, invalid-input recovery, and successful UI retry worked. Reduced motion computed to `0.00001s`. The active service worker controlled the page and `/privacy` reloaded successfully offline.
- Browser requests in normal local use stayed same-origin. Live desktop/mobile pages had one `h1`, one `main`, no axe serious/critical findings, no horizontal overflow, and only their own origin loaded. The live unauthenticated 401 console message is the exception noted above.
- Live response policy is present: CSP confines scripts/styles/images to self, `connect-src` to self plus the Sociobot license API; `X-Content-Type-Options`, `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer`, API/health `no-store`, HTML/service worker `no-cache`, and hashed JS immutable caching were observed.
- Live candidate match: SHA-256 hashes for `index-Cu11EuGV.js`, `index-BXDZOstr.css`, `hero-field-guide.webp`, and `sw.js` exactly matched the fresh local `dist/` build. This proves deployed frontend equivalence, not the backend identity.

## Limitations

- Docker, Podman, and Buildah are unavailable in this verification container, so the multi-stage image itself could not be built. Native locked release compilation and the running release binary were exercised instead.
- Lighthouse was attempted with the supplied Chromium. The launcher could not connect directly and a manually remote-debugged run crashed its browser tab, so no Lighthouse score is claimed. Asset budgets, Playwright/axe, layout, and response policy checks above completed.
- Service-worker control and offline reload passed. An actual future-worker activation could not be exercised from a single immutable script version; the shipped cache is named `capacity-sentinel-v1`.

## Reproduction

```bash
npm ci
npm test
npm run check
npm run build
npm run test:e2e
cargo build --locked --release

curl -sS https://model-capacity-sentinel.sociobot.in/health
curl -sS https://model-capacity-sentinel.sociobot.in/assets/index-Cu11EuGV.js | sha256sum
sha256sum dist/assets/index-Cu11EuGV.js
```
