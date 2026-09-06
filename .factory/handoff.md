# Capacity Sentinel repair 9 handoff

## Independent verification 9

**PASS — 0 findings and 0 untested public claims.** Independent QA reviewed
implementation `7575d65e210ae685606cb1c7c72bb1239ce15bd7`; documentation and
live `/health` identify `c4f64392faeac5837476ed0134077915621f9f95`, whose
only later change is this handoff documentation.

From a clean checkout, `npm ci`, `npm test`, `npm run check`, `npm run build`,
`npm run test:e2e -- --reporter=line`, `npm run test:claims`, locked release
build, and production dependency audit all passed. The 28 declared claims
passed, including rolling-20 metrics and session-only access-code coverage.

Fresh live desktop and phone QA passed the one-click isolated sample, reset,
offline demo reload, keyboard/focus behavior, Axe scans, legal pages, designed
404, repaired 44 px privacy email target, and HTTP/2 rate-limit allowance
(40 × 401, 60 × 429 with `Retry-After: 1`). Lighthouse measured 99/100/100/100
(performance/accessibility/best-practices/SEO). No product runtime or
deployment state was changed. See `.factory/verification-9.md` for detail.

## Result

**Repair complete.** The three verification-8 findings are fixed, all 28
declared claims pass from a clean checkout, and the repaired implementation is
live at <https://model-capacity-sentinel.sociobot.in>.

Capacity Sentinel monitors model API capacity, latency, and structured JSON
output for teams that operate applications using model APIs. The first action
is **Try it with sample data**.

## Repairs

1. Added a declared `rolling-20-observations` claim. Its API-level test inserts
   21 observations and proves the oldest failure and latency outlier do not
   affect the returned 20-record availability or p95 metrics.
2. Added a declared `session-only-access-code` claim. Its browser test proves
   the code is present in `sessionStorage`, absent from `localStorage`, gone in
   a fresh browser context, and not used for a bootstrap API request there.
3. Added “Built by Param Factory” to the application footer. The designed 404
   now has the normal skip link, Demo/Dashboard/Privacy header links, product
   line, Privacy/Terms footer links, attribution, and exact build identifier.
4. Increased the privacy email hit area to 44 CSS px. The desktop and phone
   route regression measures it.

Runtime monitoring, SQLite behavior, access control, endpoint safety, rate
limits, paid-license behavior, and demo isolation were otherwise preserved.

## Source and release identity

- Implementation commit: `7575d65e210ae685606cb1c7c72bb1239ce15bd7`
- Documentation candidate: `c3024f9726adf73de6a4b18c3a3ec2fcc5b68a53`
- Live health build: `7575d65e210ae685606cb1c7c72bb1239ce15bd7`
- Active immutable image:
  `sha256:9d8c5c1762353e0cfeeff7fad4872c7ee89a3a51d1ddb205009d458a5fd5ae2f`
- Immediate rollback revision: generated `0000013`, inactive and healthy
- Immediate rollback image:
  `sha256:ecf3333dd7585b8dff4b2227ab2aa8a9f1749748f2678e51cfd85fdca55b5282`

The later documentation commit does not change product code and was not
rebuilt or redeployed.

## Clean-checkout verification

Clean clone: `/tmp/model-capacity-sentinel-repair9.69a9IL`.

| Command | Result |
|---|---|
| `npm ci` | PASS; 134 packages, 0 vulnerabilities |
| `npm test` | PASS; 1 Vitest, 25 Rust unit/integration, 3 process tests |
| `npm run check` | PASS; Svelte 0 errors/warnings and strict Clippy |
| `npm run build` | PASS; `dist/` produced |
| `npm run test:e2e -- --reporter=line` | PASS; 24/24 desktop and phone tests |
| `npm run test:claims` | PASS; all 28 declared claim commands |
| `BUILD_SHA=7575d65… cargo build --locked --release` | PASS |
| `npm audit --omit=dev` | PASS; 0 vulnerabilities |
| ACR Docker build | PASS; immutable digest above |

Production output is 75,238 bytes JavaScript (26,893 gzip), 18,629 bytes CSS
(4,976 gzip), and 129,198 bytes for the hero WebP.

## Live verification

- `/health` returns HTTP 200, `status: ok`, and build `7575d65…`.
- The factory URL verifier passes `/` and `/demo`: correct title and language,
  one `h1`, a main landmark, complete image alternatives, labelled controls,
  and no console errors.
- Fresh 1440 × 900 and 390 × 844 contexts show the job, audience, and sample
  action before scrolling. The sample opens two realistic probes, open and
  recovered alerts, retains its demo label across reload, resets an edit, and
  returns to the separate access-code screen without any `/api/*` request.
- Live Axe scans of the demo, privacy, terms, and deliberate 404 routes have
  zero serious or critical violations on desktop and phone.
- The privacy email measures 171.89 × 44 CSS px on both viewports.
- The expected 404 returns HTTP 404 and contains its skip link, consistent
  header/footer, home action, attribution, and exact build identifier.
- A 90-request invalid-access burst returned 43 HTTP 401 and 47 HTTP 429.
  Every 429 had a positive `Retry-After`; a request after 2.1 seconds returned
  401 and `/health` remained 200.
- Mobile Lighthouse: performance 100, accessibility 100, best practices 100,
  SEO 100; LCP 1.7 s, total blocking time 40 ms, CLS 0.

Evidence is under `/work/.evidence/repair-9/`. The catalog description was
copied to `/work/.evidence/catalog-description.txt`, and the pending Atlas
offer metadata is `/work/.evidence/billing-offer.json`.

## Deployment state

- Provisioning: `Succeeded`; running status: `Running`.
- Revision mode: `Single`.
- Generated revision `sf-model-capacity-sentinel--0000014` is the only active
  revision; it is healthy and provisioned.
- Exactly one replica is running and ready with zero restarts.
- Traffic remains `latestRevision: true`, weight 100.
- Scale remains one minimum and one maximum replica.
- `sf-model-capacity-sentinel-data` remains mounted at `/data`.

No manually named revision or traffic target was created.

## Known dependency

Atlas remains a $39 one-time paid add-on for the 365-day comparison view.
Sociobot billing registration is still pending, so the live site honestly
withholds checkout while retaining the tested license return, restore,
verification, caching, invalid/revoked handling, and paid comparison behavior.
The free monitoring core remains available. No provider credential was
invented; real endpoint behavior is covered by local fixtures.
