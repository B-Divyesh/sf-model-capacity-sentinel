# Independent verification 4 — FAIL

Verified on 2026-08-28 against candidate commit
`53e982e1c063047c40bbdc2f6dc8385fb4a2ee18` and the live deployment
<https://model-capacity-sentinel.sociobot.in>.

## Decision

**FAIL.** The previous deployment-only build-identity failure is repaired: the
live backend and frontend can now be tied to this exact candidate. The product
is otherwise buildable and the normal, boundary, accessibility, privacy, PWA,
and responsive checks below passed. It nevertheless fails the mandatory
backend rate-limiting contract.

## Release-blocking defect

### High — API rate limiting is incomplete and 429 lacks `Retry-After`

Fresh local HTTP evidence against the candidate's running server:

- 100 concurrent authenticated `GET /api/summary` requests all returned 200;
  reads have no rate limit at all.
- A burst of 65 authenticated `POST /api/probes` requests reached 429 at
  request **61** (60 returned 422 for the deliberately invalid body; 5
  returned 429). None of the 429 responses contained `Retry-After`.
- The implementation is one global in-memory `VecDeque` of writes. It is not
  per client IP and does not inspect the required first `X-Forwarded-For` hop.

The applicable acceptance contract requires every server-side endpoint
(apart from health) to be rate limited per client and requires every 429 to
include `Retry-After`. This is a release blocker even though the write burst
does eventually return 429.

Required remediation: use a per-client limiter keyed from the first
`X-Forwarded-For` address (with safe socket fallback), cover API reads as well
as writes, and emit an accurate `Retry-After` header on each 429. Add route
tests for the header and both read/write limits.

## Passing evidence

### Clean local checkout and quality gates

- Checkout was clean and exactly `53e982e1c063047c40bbdc2f6dc8385fb4a2ee18`.
  `npm ci` completed successfully with 0 reported audit vulnerabilities.
- `npm test` passed: 3 Vitest tests, 7 Rust unit/integration tests, and 1
  startup-process test.
- `npm run check` passed: Svelte check had 0 errors and 0 warnings; strict
  `cargo clippy -- -D warnings` passed.
- `npm run build` passed and produced `dist/`.
- `npm run test:e2e -- --reporter=line` passed all 6 tests across desktop and
  390 x 844 mobile, including populated-dashboard axe scans.
- `BUILD_SHA=53e982e1c063047c40bbdc2f6dc8385fb4a2ee18 cargo build --locked
  --release` passed. Its locally started release binary returned that same SHA
  from `/health`.

### Product behavior and privacy

- Existing integration coverage exercised the successful JSON-invariant path,
  repeated 429 classification and attributed availability alert after two
  runs, update/export/delete lifecycle, and legacy prompt migration.
- Independent process checks confirmed: unauthenticated summary 401; a valid
  public-endpoint canary create 201; loopback target rejected 400; out-of-range
  interval rejected 400; the API summary omitted the canary and API key; raw
  SQLite/WAL strings contained neither supplied secret; and generated
  `master.key` mode was 0600.
- The no-environment startup process test passed, including generated secret
  provenance logging and persistence behavior. No production prompts are
  solicited by default; UI copy makes the synthetic-only boundary and the
  non-semantic-quality limitation clear.
- Browser traffic in fresh desktop and mobile live visits was same-origin
  only. The only configured external connection is the optional Sociobot
  license API, permitted by CSP; no third-party fonts, scripts, analytics, or
  sign-in provider are used.

### Browser, accessibility, PWA, performance, and policies

- Independent live browser checks at 1440 x 900 and 390 x 844 found one `h1`,
  one `main`, no horizontal overflow, no browser console errors, and no axe
  serious/critical findings. The local populated mobile check additionally
  confirmed keyboard focus as `rgb(23, 95, 132) solid 3px`, no overflow, and
  no console/page errors.
- Keyboard dialog focus/Escape and access-code Enter recovery are covered by
  the passing browser suite. Reduced-motion computed transition duration was
  `0.00001s`.
- The service worker controlled the page; `registration.update()` succeeded,
  and `/privacy` reloaded successfully offline.
- Fresh bundles meet the stated static budgets: JavaScript 67,533 bytes raw /
  25,450 gzip (under 200 KB), CSS 16,991 / 4,660 gzip (under 50 KB), and hero
  WebP 129,198 bytes (under 300 KB). No webfonts are shipped.
- Live response policy was confirmed: CSP restricting script/style/image to
  self and `connect-src` to self plus the Sociobot license API,
  `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`,
  `Referrer-Policy: no-referrer`, API/health `no-store`, HTML/service worker
  `no-cache`, and one-year immutable caching for hashed JS/CSS. Cross-origin
  API response did not grant CORS access.

### Live candidate match

`GET https://model-capacity-sentinel.sociobot.in/health` returned:

```json
{"build":"53e982e1c063047c40bbdc2f6dc8385fb4a2ee18","status":"ok"}
```

Fresh SHA-256 comparisons matched exactly:

| Artifact | SHA-256 |
|---|---|
| `assets/index-BrAiOFsX.js` | `824a6477eb34c5ed3b1d90471d1bbc9d9a325b6c7fa5e72bfa8305dae8869fe5` |
| `assets/index-BXDZOstr.css` | `3e8cba760e347bf4040dc65da060e95bd58f7220c15602b7fa3a6cd21a9a8162` |

Together with the immutable live health identity, this confirms the deployed
frontend and backend match the tested candidate.

## Limitations

- Docker is not installed in this verification container, so the Docker image
  could not be built here. Native locked release compilation, runtime startup,
  and the live container identity were verified.
- No Lighthouse binary is installed. Bundle budgets, responsive rendering,
  browser console/error checks, accessibility, and service-worker behavior
  were independently measured; no fresh Lighthouse score is claimed.

## Reproduction

```bash
npm ci
npm test
npm run check
npm run build
npm run test:e2e -- --reporter=line
BUILD_SHA=53e982e1c063047c40bbdc2f6dc8385fb4a2ee18 cargo build --locked --release

curl -sS https://model-capacity-sentinel.sociobot.in/health
curl -sS https://model-capacity-sentinel.sociobot.in/assets/index-BrAiOFsX.js | sha256sum
sha256sum dist/assets/index-BrAiOFsX.js
```

For the blocker, start a local server with a known access token and burst 65
authenticated invalid `POST /api/probes` requests. Requests 61–65 return 429
without `Retry-After`; burst `GET /api/summary` returns no 429 at all.
