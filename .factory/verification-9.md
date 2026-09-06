# Verify model API capacity monitoring — verification 9

Date: 2026-09-06  
Live URL: <https://model-capacity-sentinel.sociobot.in>

## Verdict

**PASS — 0 findings and 0 untested public claims.**

No product code, deployment configuration, revision, traffic rule, storage,
secret, or project data was changed during this independent verification.

## Job, audience, and first action before scrolling

- Job: monitor model API capacity early, including failures, latency, and JSON
  output shape.
- Audience: teams running model APIs that need early evidence before users
  report an issue.
- First action: **Try it with sample data**.

Fresh 1440 × 900 desktop and 390 × 844 phone contexts showed the H1, audience
sentence, and action above the fold. One click opened the populated `/demo`
workspace.

## Candidate and live provenance

| Identity | Value | Result |
|---|---|---|
| Implementation candidate reviewed | `7575d65e210ae685606cb1c7c72bb1239ce15bd7` | PASS |
| Documentation / repository HEAD | `c4f64392faeac5837476ed0134077915621f9f95` | PASS |
| Live `/health` build | `c4f64392faeac5837476ed0134077915621f9f95` | PASS |

`git diff 7575d65..c4f6439` changes only `.factory/handoff.md`; the later
documentation commit does not change the implementation. Live health therefore
identifies the documentation build while serving the reviewed implementation.

## Clean-checkout verification

Fresh clone: `/tmp/mcs-v9-clean.SYom3K` at `c4f6439…`.

| Command | Result |
|---|---|
| `npm ci` | PASS — 134 packages; 0 reported vulnerabilities |
| `npm test` | PASS — 1 Vitest, 25 Rust unit/integration, and 3 process tests |
| `npm run check` | PASS — Svelte 0 errors/warnings; Clippy warnings denied |
| `npm run build` | PASS — `dist/` produced; 75.23 KB JS and 18.62 KB CSS before gzip |
| `npm run test:e2e -- --reporter=line` | PASS — 24/24 desktop and phone tests |
| `npm run test:claims` | PASS — all 28 declared commands and exact-tag audit |
| `BUILD_SHA=7575d65… cargo build --locked --release` | PASS |
| `npm audit --omit=dev` | PASS — 0 vulnerabilities |

The two repaired public promises are now complete tagged claims:
`rolling-20-observations` inserts 21 records and excludes the oldest failure
and latency outlier; `session-only-access-code` proves session storage only,
no local-storage copy, and an empty fresh session. Each of the 28 manifest
entries has exactly one test tag and was run from the clean checkout. There
are no unlisted, relied-on public promises found in the landing page, legal
pages, or README that lack applicable claim coverage.

## Live product evidence

- Fresh desktop and phone demo flows displayed two realistic probes, a 429
  capacity alert, and a recovered latency alert. The persistent demo label
  survived reload; an edited sample reset to shipped data; **Start for real**
  returned to the separate access-code screen. No cross-origin or `/api/*`
  request occurred while using the demo.
- Service worker registration and update check succeeded. After the controlled
  first `/demo` visit, an offline reload kept the Demo title and sample label.
- Axe found zero violations on `/`, `/demo`, `/privacy`, `/terms`, and the
  designed 404 on desktop and phone. Keyboard starts at the skip link; the
  privacy email has a visible 3 px focus outline and measures 171.89 × 44 CSS
  px on a 390 px phone.
- `/privacy`, `/terms`, and the deliberate unknown path each have one H1 and
  one main landmark. The expected unknown route returned HTTP 404 with a
  complete skip link, header links, Privacy/Terms footer links, attribution,
  exact build ID, and a way home. The browser's expected network console entry
  for navigating to an HTTP 404 is not a page defect; the designed document
  itself has no script or CSP error.
- `/`, `/demo`, `/privacy`, `/terms`, static metadata assets, robots, and
  sitemap returned 200. HTTPS headers include CSP, frame denial, no-sniff,
  and `Referrer-Policy: no-referrer`; hashed assets have immutable caching.
- A single HTTP/2 connection sent 100 invalid access requests. 40 returned
  401 and 60 returned 429, each with `Retry-After: 1`. A request after a
  three-second recovery window returned 401; `/health` remained 200.
- The project access boundary, endpoint validation, encryption at rest,
  restart persistence, non-root startup, invalid/boundary inputs, failure
  classification, scheduled probes, output/token caps, deletion, and alert
  recovery all passed their temporary SQLite/local-fixture claim tests.
- Mobile Lighthouse independently measured performance **99**,
  accessibility **100**, best practices **100**, and SEO **100**; LCP was
  1.849 s, TBT 93 ms, CLS 0, and transfer 162,710 bytes. This meets the
  required performance threshold. The one-point difference from the prior
  report's 100 is a measurement result, not a public product claim.

No provider credential was available or invented. Provider behavior was
exercised with repository local fixtures only.

## Earlier finding disposition

| Earlier finding | Current disposition |
|---|---|
| Review 1 F1: missing sample | Fixed — one-click, populated, isolated sample, persistent label, reset, and start-for-real separation pass. |
| Review 1 F2 / Review 2 F1: incomplete claims | Fixed — all 28 declared claims pass and the two verification-8 promises are now manifested and tested. |
| Review 1 F3: inert edit | Fixed — pointer and keyboard edit tests pass; history preservation passes. |
| Review 1 F4 / Verification 4: rate limiting | Fixed — live HTTP/2 burst produces 429 plus `Retry-After`; recovery and health exemption pass. |
| Review 1 F5: broken checkout | Fixed — checkout remains honestly unavailable pending registration; restore/verification lifecycle is tested. |
| Review 1 F6: unclear first screen | Fixed — job, audience, and sample action are above the fold on desktop and phone. |
| Review 1 F7: routes and metadata | Fixed — legal titles/metadata, assets, designed 404, and completed header/footer/skip structure pass. |
| Review 1 F8: pinned Rust base | Fixed — clean locked release build passes. |
| Review 1 F9: small touch targets | Fixed — repaired privacy email is 44 px high; route tests and live phone measurement pass. |
| Verification 1: public writes, plaintext, populated Axe, caching | Fixed — access coverage, encrypted storage, privacy egress, Axe, and immutable asset checks pass. |
| Verification 3: build identity, console, startup | Fixed — health build identity and clean rendered pages pass; generated startup material is covered locally. |
| Verification 6: outage/restoration | Historical — live health and all independent runtime checks pass; no recovery experiment was performed. |
| Verification 8 F1: two untested promises | Fixed — `rolling-20-observations` and `session-only-access-code` pass. |
| Verification 8 F2: incomplete footer/404 structure | Fixed — live footer and designed 404 contain every required element. |
| Verification 8 F3: 20 px privacy email target | Fixed — 44 px live measurement passes. |

## Evidence

The independent phone landing screenshot and Lighthouse JSON are under
`/work/.evidence/verification-9-phone-landing.png` and
`/work/.evidence/verification-9-lighthouse.json`. This report is also copied
to `/work/.evidence/qa-report.md`; its machine-readable companion is
`/work/.evidence/qa-result.json`.
