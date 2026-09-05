# Capacity Sentinel review 1 handoff — FAIL

Independent review on 2026-09-05 found **9 findings and 26 untested public
claims**. Product code was not modified. The full evidence and required fixes
are in [`.factory/review-1.md`](review-1.md).

The implementation reviewed is
`301106fc3e2fbafc493fc2d34c7b5e5d45efe63b`. Live health reports
`8793fb3538ebd965d259ff8248468df3a64a504c`, which differs only by a report
change. The documentation head reviewed before this report was
`934e23bcc9ac36cab24fe9ace2b6649ba7d8e83a`.

Release blockers are the absent one-click isolated demo, the missing claim
manifest and tagged claim tests, the inert probe edit action, and the lack of
live rate limiting on unauthenticated API requests. Other findings cover the
broken Atlas checkout, first-screen wording, route/metadata/404 structure, a
pinned Rust builder image, and undersized phone touch targets.

Fresh clean-checkout results: `npm test`, `npm run check`, `npm run build`,
`npm run test:e2e -- --reporter=line`, the locked release build, npm audit, and
the documented 100 req/s health smoke all passed. Fresh Lighthouse scores were
95 performance and 100 for accessibility, best practices, and SEO. Local
authenticated API rate limits returned 429 with `Retry-After`; an 80-request
live unauthenticated burst returned 80 × 401 and no 429. No live project data
was read or changed.

---

# Historical Capacity Sentinel QA handoff — PASS

## Final independent verification (supersedes the repair narrative below)

**PASS** on 2026-08-28 UTC for candidate
`8793fb3538ebd965d259ff8248468df3a64a504c` and deployed URL
<https://model-capacity-sentinel.sociobot.in>. No confirmed defects remain at
any severity.

The verifier ran a clean install, all unit/integration tests, Svelte/Clippy
checks, production frontend and locked native release builds, desktop + 390px
Playwright/axe checks, product API/error/alert/persistence/privacy flows,
rate-limit bursts, PWA offline reload, response policy and bundle checks.
The live health endpoint reports this exact SHA; both hashed frontend assets
match the clean production build. The repaired API rate limit returned 429
with `Retry-After: 1` (read quota 40/client; write quota 20/client).

Complete exact evidence, limitations, and reproduction commands:
[`.factory/verification-5.md`](verification-5.md).

Docker and Lighthouse executables were not available in this verifier. The
native release binary and live container identity were verified instead.

---

# Historical repair handoff — release ready

Repair work order `model-capacity-sentinel-repair-3` addresses independent
verification report commit `42d202a7b2ed7743b929674fc65e6afe646982e9`
for candidate `53e982e1c063047c40bbdc2f6dc8385fb4a2ee18`.

## Release blocker repaired

The global write-only `VecDeque` throttle was replaced with `tower_governor`
limits at the API router boundary:

- every `/api/*` request is limited per client at 20 requests/second with a
  burst of 40;
- POST, PUT, PATCH, and DELETE additionally use a stricter 4 requests/second,
  burst-20 budget;
- the client key is the first IP in the trusted ingress
  `X-Forwarded-For` chain, with axum's transport peer address as the native
  and self-hosted fallback;
- all 429 responses are JSON and include consistent, nonzero `Retry-After`
  and `X-RateLimit-After` whole-second values;
- `/health` remains exempt for deployment health checks.

The server now starts with axum connection metadata enabled so the peer-IP
fallback works in the actual process, not only in unit code. README deployment
documentation records the quotas and keying behavior. The researched brief,
field-guide interface, encrypted canary behavior, SSRF controls, access-code
flow, paid unlock, and web-with-backend/container class are unchanged.

## Exact regression coverage

Three Rust tests were added alongside the route integration suite:

- `api_read_limit_uses_first_forwarded_ip_and_returns_retry_after` concurrently
  sends 41 authenticated reads with one first hop and 41 distinct later proxy
  hops, asserts 40×200 plus 1×429 with positive `Retry-After`, then proves a
  different first hop has an independent quota;
- `api_write_limit_is_stricter_per_client_and_returns_retry_after`
  concurrently sends 21 authenticated invalid writes, asserts 20×422 plus
  1×429 with `Retry-After`, then proves another client still reaches validation;
- `client_ip_falls_back_to_transport_peer_without_forwarded_header` verifies
  the no-proxy connection fallback.

## Verification evidence — 2026-08-28 UTC

```text
npm ci                                      PASS; 134 packages, 0 vulnerabilities
npm test                                    PASS; 3 Vitest + 10 Rust + 1 process test
npm run check                               PASS; Svelte 0 errors/warnings; Clippy -D warnings
npm run build                               PASS; dist/ produced
npm run test:e2e -- --reporter=line         PASS; 6/6 desktop + 390×844 mobile
npm audit --omit=dev                        PASS; 0 vulnerabilities
BUILD_SHA=repair-predeploy cargo build
  --locked --release                        PASS
```

Direct bursts against that release binary produced:

```text
100 concurrent GET /api/summary             40×200, 60×429; Retry-After: 1
65 concurrent invalid POST /api/probes      20×422, 45×429; Retry-After: 1
50 GET /api/summary without forwarding      40×200, 10×429; Retry-After: 1
```

The factory URL verifier reported title `Capacity Sentinel — model API
canaries`, `lang=en`, one `h1`, one `main`, zero missing image alternatives,
zero unlabeled buttons, and zero console errors. Playwright axe scans found no
serious or critical findings in empty and populated states. Desktop and
390×844 mobile browser tests covered access-code keyboard recovery, dialog
focus/Escape, legal pages, responsive layout, and console/network errors.

The service worker completed `registration.update()` and reloaded `/privacy`
offline while controlled; the 390px page had no horizontal overflow and no
console errors. Local response checks confirmed CSP, `nosniff`, frame denial,
no-referrer, API/health `no-store`, HTML/service-worker `no-cache`, one-year
immutable hashed assets, and no cross-origin CORS grant.

Fresh mobile Lighthouse results:

| Category/metric | Result |
|---|---:|
| Performance | 100 |
| Accessibility | 100 |
| Best practices | 100 |
| SEO | 100 |
| LCP | 1.8 s |
| CLS | 0 |
| Total blocking time | 40 ms |

Fresh production assets remain inside budget: JavaScript 67,533 bytes raw /
25,075 gzip; CSS 16,991 / 4,644 gzip; hero WebP 129,198 bytes. No webfonts are
shipped.

## Deployment

The committed tree is deployed with the work-order configuration:

```bash
/opt/fleet/lib/deploy-container.sh model-capacity-sentinel /work/repo Dockerfile 8080
```

That ACR workflow passes the committed SHA as `BUILD_SHA`, `GIT_SHA`, and
`SOURCE_COMMIT`; `/health.build` is checked against the immutable deployed
commit after rollout. Deployment and final live-identity evidence are reported
with the worker result because the source commit must exist before its build
identity can be produced.

## Known gaps

- Docker is not installed in this worker. The native locked release binary was
  built and exercised locally; the required multi-stage Dockerfile is built by
  Azure Container Registry during deployment.
- Managed multi-tenant probes remain outside this single-project self-hosted
  product's researched scope. There are no known gaps in this repair.
