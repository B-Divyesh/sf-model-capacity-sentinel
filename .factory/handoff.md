# Capacity Sentinel verification handoff — FAIL

Independent verification work order `model-capacity-sentinel-verify-3` tested
commit `1a31ab744b4f76184b70f6f8ee36b16b0dfc403b` against
<https://model-capacity-sentinel.sociobot.in> on 2026-08-28.

**FAIL — do not release this deployment.** The deployed frontend is exactly
the candidate build, but `/health` returns `{"build":"unknown","status":"ok"}`
rather than the tested commit. The live backend is therefore not immutably
identifiable as the candidate.

Required next step: rebuild/redeploy with `BUILD_SHA` set to
`1a31ab744b4f76184b70f6f8ee36b16b0dfc403b`, then verify the live health body.
Also prevent the intentional unauthenticated bootstrap 401 from appearing as a
browser console error, and add the required startup log that says which
configuration/secrets were generated versus supplied without disclosing them.

Full fresh evidence, commands, severity, tested URL, and limitations are in
`.factory/verification-3.md`. The repair handoff below is historical context
and is superseded by this FAIL decision.

---

# Historical repair handoff

Repair work order `model-capacity-sentinel-repair-1`, based on verifier report
for candidate `8bdec478d62084ccdf7fcd5bbfe46cfe6f21e8b9`.

## Release blockers repaired

- All `/api/*` routes now require a constant-time checked, per-project bearer
  access code. The browser prompts for it when needed and keeps it only in
  `sessionStorage`; summaries, history, CSV export, and every write are
  protected. Set `SENTINEL_ACCESS_TOKEN` in the container deployment secret.
  Without it, a random 32-byte token is generated as mode-0600
  `DATA_DIR/access.token` for local use.
- Probe endpoints must resolve only to public internet addresses. Loopback,
  private, carrier-grade NAT, link-local, multicast, unspecified,
  documentation, and reserved address ranges are rejected. The address set is
  rechecked for every probe and pinned into the outbound client; redirects are
  disabled.
- Added migration `0002_encrypt_canaries.sql`. Existing plain canaries are
  encrypted with the existing AES-256-GCM secret, their legacy text is blanked,
  and the database is securely compacted/WAL-truncated. New canaries are
  encrypted from the start and no read API returns canary text.
- Replaced the invalid labelled plain trend `div` with a valid named `img`
  role, and added an axe regression on the populated dashboard for both
  desktop and 390px mobile.
- `/health` now reports the runtime `BUILD_SHA` set by the Docker build; Docker
  declares it in the final image. HTML/service worker are `no-cache`, hashed
  JS/CSS are `public, max-age=31536000, immutable`, and APIs/health are
  `no-store`.

## Verification evidence

Run in a clean dependency install:

```text
npm ci                                      # 0 audit vulnerabilities
npm test                                    # 2 Vitest + 6 Rust tests passed
npm run check                               # Svelte 0 errors/0 warnings; Clippy -D warnings passed
npm run build                               # dist produced
npm run test:e2e                            # 4/4 passed: desktop + 390×844 mobile
npm audit --omit=dev                        # 0 vulnerabilities
```

The Playwright suite exercises the empty/legal/keyboard dialog flow and a
seeded populated dashboard with axe 4.10.2; serious and critical findings are
zero in both viewports. Rust integration coverage now asserts:

- unauthenticated API access is 401;
- loopback probe URLs are rejected;
- synthetic canaries are blank in the plaintext column, encrypted at rest, and
  absent from `/api/summary`;
- a simulated legacy plaintext record is encrypted and scrubbed on migration.

Manual release-HTTP smoke against the local production frontend confirmed:

```text
GET /health                                  200
GET /api/summary (no Authorization)          401
GET /api/summary (valid Bearer token)         200
POST /api/probes -> http://127.0.0.1/...      400
GET /assets/index-Cu11EuGV.js                Cache-Control: public, max-age=31536000, immutable
```

Production asset sizes: JS 67,499 bytes raw (25,440 gzip), CSS 16,991 bytes
raw (4,660 gzip), hero WebP 129,198 bytes. Docker is not installed in this
worker, so the multi-stage image was not built locally; native release checks
and the Dockerfile build contract were checked instead.

## Deploy/run

The deployment remains the required Rust/Axum + SQLite container on port 8080.
The factory container build must use this immutable release identifier and set
the project-secret environment variable:

```bash
docker build --build-arg BUILD_SHA=$(git rev-parse HEAD) -t capacity-sentinel .
docker run -p 8080:8080 -e SENTINEL_ACCESS_TOKEN='<32+ character secret>' -v sentinel-data:/data capacity-sentinel
```

Verify the release after deployment with `curl -sS https://model-capacity-sentinel.sociobot.in/health`; `build` must equal the immutable release commit. Do not expose the generated local access token in logs, URLs, or source control.

## Known limits

- This remains a single-project self-hosted runner. The project access code is
  the isolation boundary; a managed multi-tenant offering needs account-level
  projects and per-user authorization rather than sharing one access code.
- Public provider endpoints are intentional for this release. Private/VPC
  providers need a future agent/allowlist architecture rather than weakening
  SSRF protection.
