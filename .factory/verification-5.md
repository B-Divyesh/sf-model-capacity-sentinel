# Independent verification 5 — PASS

Verified on 2026-08-28 UTC from a clean detached checkout of candidate
`8793fb3538ebd965d259ff8248468df3a64a504c`, against
<https://model-capacity-sentinel.sociobot.in>.

## Decision

**PASS.** Capacity Sentinel meets the researched smallest useful product:
it stores synthetic canaries and credentials encrypted locally, probes public
OpenAI-compatible endpoints, attributes capacity/latency/output-shape failure,
keeps operational evidence locally, and opens vendor/model-specific alerts.
The prior release-blocking rate-limit defect is repaired. The live backend
identifies this exact candidate and its JavaScript/CSS assets hash-identically
to the clean production build.

## Fresh quality-gate evidence

```text
clean checkout                         PASS  detached HEAD 8793fb3538ebd965d259ff8248468df3a64a504c
npm ci                                 PASS  134 packages; npm audit reported 0 vulnerabilities
npm test                               PASS  3 Vitest tests; 10 Rust unit/integration; 1 startup-process test
npm run check                          PASS  Svelte: 0 errors/warnings; cargo clippy -- -D warnings
npm run build                          PASS  production dist/ built
npm run test:e2e -- --reporter=line    PASS  6/6 Playwright tests (desktop and 390 x 844 mobile)
BUILD_SHA=<candidate> cargo build
  --locked --release                   PASS  native release binary
```

The browser suite includes empty/access-code recovery, keyboard dialog focus
and Escape, legal pages, populated UI, and axe scans. Fresh live axe scans at
1440px and 390px found **zero serious/critical violations**. Fresh local
desktop/mobile checks found one `h1`, one `main`, no horizontal overflow, no
console/page errors, no initial external request, and a visible
`rgb(23, 95, 132) solid 3px` keyboard focus outline. With reduced motion,
the chevron transition was `0.00001s`.

## End-to-end product evidence

- A normal public-endpoint canary (`https://httpbin.org/post`) was accepted
  with `201`; an execution returned the expected `invalid_json` outcome when
  that endpoint's non-OpenAI response lacked `choices[0].message.content`.
- Invalid recovery paths behaved correctly: unauthenticated summary `401`,
  interval `0` rejected `400`, and a `127.0.0.1` endpoint rejected `400` as
  non-public.
- Two runs against `https://httpbin.org/status/429` produced attributed
  `capacity` observations and an open availability alert after the second
  run. Three runs against `https://httpbin.org/delay/1` measured
  1037–1166 ms against a 100 ms target and opened a latency alert at p95
  1166 ms.
- Restarting the release binary with the same SQLite directory retained 3
  probes, 6 observations and 4 open alerts. Deleting the injected-capacity
  specimen returned `204` and removed its observations and alerts.
- A known API key and canary text were absent from API summary and from raw
  SQLite/WAL string scans. Generated `master.key` was mode `0600`.

## Rate limiting

The native release binary was tested with an access token and a fixed first
`X-Forwarded-For` hop. A 60-way concurrent read burst returned 50 `200` and
10 `429` (some tokens replenished during the 0.8-second client burst); every
429 included `Retry-After: 1`. A 25-way invalid write burst returned 20
validation responses and 5 `429`, so the observed write threshold was 20
requests/client; every 429 included `Retry-After: 1` and
`X-RateLimit-After: 1`. The included exact integration checks establish the
read burst threshold as 40/client and the write burst threshold as 20/client,
and verify independent first-forwarded-IP buckets plus peer-IP fallback.

The public deployment does not disclose a project access code, so an
unauthenticated live 60-way `/api/summary` burst correctly returned 60 `401`.
The identical live build identity and assets, plus the authenticated local
release-binary evidence, verify the implemented policy without exposing a
live project credential.

## Privacy, PWA, performance, and response policy

- Local and live headers: strict self CSP (only optional
  `https://api.sociobot.in` license connection), `nosniff`, `DENY`,
  `no-referrer`; API/health `no-store`; HTML/service worker `no-cache`; and
  hashed JS/CSS one-year immutable caching. A cross-origin API request did
  not receive a CORS grant. No CDN fonts, scripts, analytics, or sign-in
  provider are shipped. The optional license handoff uses Sociobot only.
- Service worker was controlling the page; `registration.update()` succeeded;
  `/privacy` reloaded offline. The fresh normal page made no external browser
  requests on either desktop or mobile.
- Production output: JS 67,533 bytes raw / 25,066 gzip; CSS 16,991 / 4,631
  gzip; hero WebP 129,198 bytes; no fonts. All are within stated budgets.
- `GET /health` from the live URL returned
  `{"build":"8793fb3538ebd965d259ff8248468df3a64a504c","status":"ok"}`.
  Local/live SHA-256 matched for `index-BrAiOFsX.js`
  (`824a6477eb34c5ed3b1d90471d1bbc9d9a325b6c7fa5e72bfa8305dae8869fe5`)
  and `index-BXDZOstr.css`
  (`3e8cba760e347bf4040dc65da060e95bd58f7220c15602b7fa3a6cd21a9a8162`).

## Defects

No confirmed release-blocking, high, medium, or low defects.

## Verification-environment limitations

- Docker is not installed in this disposable verifier, so its local Docker
  image build could not be invoked. The locked native release build started
  successfully with the intended SHA, and the deployed container reports
  that exact SHA.
- No Lighthouse binary was installed. Browser accessibility, responsiveness,
  console/error state, PWA behavior, policies, and all explicit bundle-size
  budgets were independently measured; no new Lighthouse score is claimed.
- A live healthy OpenAI-compatible endpoint requires a user-supplied valid
  provider credential. The accepted normal configuration path and all
  externally reproducible error/alert paths were exercised; parser and
  invariant primitives are covered by the passing Rust suite.

## Reproduce

```bash
npm ci
npm test
npm run check
npm run build
npm run test:e2e -- --reporter=line
BUILD_SHA=8793fb3538ebd965d259ff8248468df3a64a504c cargo build --locked --release
curl -sS https://model-capacity-sentinel.sociobot.in/health
```
