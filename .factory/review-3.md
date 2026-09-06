# Review 3 — Monitor model API capacity early

Date: 2026-09-06  
Live URL: <https://model-capacity-sentinel.sociobot.in>

## Verdict

**PASS — 0 findings and 0 untested public claims.**

This was a review-only pass. No product code, deployment configuration,
revision, traffic rule, storage, secret, or product data was changed.

## Job, audience, and first action before scrolling

- Job: monitor model API capacity early, including failures, latency, and JSON
  output shape.
- Audience: teams running model APIs that need early evidence before users
  report an issue.
- First action: **Try it with sample data**.

Fresh 1440 × 900 desktop and 390 × 844 phone contexts showed the H1, audience
sentence, and primary action above the fold at scroll position zero. The action
opened the populated sample in one click.

## Candidate and live provenance

| Identity | Value | Result |
|---|---|---|
| Implementation candidate reviewed | `7575d65e210ae685606cb1c7c72bb1239ce15bd7` | PASS |
| Repository documentation SHA | `6eecbb6bbaa4722a0f742bdb42bb1d7fa3b18ed3` | PASS |
| Live `/health` build | `c4f64392faeac5837476ed0134077915621f9f95` | PASS |

The live build identifies the report-only handoff revision recorded by
verification 9. `git diff 7575d65..6eecbb6` contains only
`.factory/handoff.md` and `.factory/verification-9.md`; there is no later
product-code change. The public health endpoint returned HTTP 200 with
`status: ok` and the live build above.

## Clean-checkout verification

Fresh detached checkout: `/tmp/mcs-review3-clean.TMgOOS` at
`6eecbb6bbaa4722a0f742bdb42bb1d7fa3b18ed3`.

| Command | Result |
|---|---|
| `npm ci` | PASS — 134 packages; 0 reported vulnerabilities |
| `npm test` | PASS — 1 Vitest, 25 Rust unit/integration, and 3 process tests |
| `npm run check` | PASS — Svelte 0 errors/warnings; Clippy warnings denied |
| `npm run build` | PASS — `dist/` produced |
| `npm run test:e2e -- --reporter=line` | PASS — 24/24 desktop and phone tests |
| `npm run test:claims` | PASS — all 28 declared claim commands |
| `BUILD_SHA=7575d65… cargo build --locked --release` | PASS |
| `npm audit --omit=dev` | PASS — 0 vulnerabilities |

The production build is 75,238 bytes JavaScript (27.29 KB gzip) and 18,629
bytes CSS (4.99 KB gzip). The clean claim runner executed 28 commands for the
28 manifest entries and exited 0. The two formerly uncovered promises have
complete evidence: the rolling metric test inserts 21 observations, and the
session test verifies session-only access-code storage in a fresh browser
context.

## Live product evidence

- Fresh desktop and phone visits had the correct title, one H1, one main
  landmark, no initial scroll, and no console errors.
- The sample showed two realistic provider/model probes, a capacity alert,
  and a recovered latency alert. Its persistent **Demo — sample data, nothing
  is saved** label survived an offline reload after service-worker control.
- On a fresh phone context, editing `Europe structured output` to a review-only
  name changed the sample. **Reset demo** removed that change. **Start for
  real** returned to the separate access-code screen. The complete flow made
  no `/api/*` request and no cross-origin request.
- Live Axe scans on `/`, `/demo`, `/privacy`, `/terms`, and the designed
  `/not-a-real-route` page found zero violations. The deliberate HTTP 404 had
  its expected network-console entry only; its rendered page had a title, H1,
  main landmark, skip link, full header/footer, attribution, build ID, and a
  way home.
- At 390 px, `privacy@sociobot.in` measured 171.89 × 44 CSS px. Its focus and
  touch target meet the required baseline.
- `/`, `/demo`, `/privacy`, `/terms`, `/robots.txt`, `/sitemap.xml`, favicon,
  touch icon, and social image returned 200; the unknown route returned the
  deliberate 404. Hashed JavaScript has `Cache-Control: public, max-age=31536000, immutable`.
- A 100-request concurrent invalid-access burst produced 96 × 401 and 4 ×
  429. Every 429 carried `Retry-After: 1`; after three seconds the same client
  returned to 401, while `/health` remained 200.
- The authoritative verification-9 Lighthouse measurement remains 99
  performance, 100 accessibility, 100 best practices, and 100 SEO. This
  review independently reran the browser, accessibility, offline, and
  rate-limit checks above.

## Earlier finding disposition

| Earlier finding family | Current disposition |
|---|---|
| Review 1 F1: no one-click sample | Fixed — live populated sample, label, reset, and real-project separation pass. |
| Review 1 F2 / Review 2 F1: incomplete public claims | Fixed — 28 manifest commands passed; no unlisted relied-on promise was found in the reviewed landing, legal, or README copy. |
| Review 1 F3: inert edit control | Fixed — clean keyboard/pointer test and live sample edit/reset pass. |
| Review 1 F4 / Verification 4: missing effective rate limit | Fixed — fresh live burst returned 429 with positive `Retry-After` and recovered. |
| Review 1 F5: broken checkout | Fixed — checkout is honestly withheld while registration is pending; tested restore and invalid/revoked license behavior remains available. |
| Review 1 F6: unclear first screen and metaphor copy | Fixed — the job, audience, and sample action are plain and above the fold on both viewports. |
| Review 1 F7: routes, metadata, and empty 404 | Fixed — legal route titles, metadata assets, designed 404, and navigation structure pass. |
| Review 1 F8: pinned Rust image | Fixed — Dockerfile uses `rust:1-slim`; locked native release build passed. |
| Review 1 F9 / Verification 8 F3: undersized targets | Fixed — live privacy email is 44 px high; Axe and phone checks pass. |
| Baseline verification: public writes, plaintext canaries, populated Axe, and caching | Fixed — access boundary, encrypted-storage/privacy claim evidence, live Axe, and immutable asset caching pass. |
| Verification 3: health identity, first-load console error, startup record | Fixed — public health returns an immutable build, fresh normal pages have no errors, and clean startup coverage passes. |
| Verification 6: public outage | Fixed — the live homepage and health endpoint responded throughout this review. |
| Verification 8 F1: rolling-20 and session-only promises untested | Fixed — both are declared and their complete commands passed. |
| Verification 8 F2: incomplete footer and 404 skeleton | Fixed — live footer and 404 structure include all required elements. |

## Evidence

The clean-checkout logs are under `/tmp/mcs-review3-*`. The independent
verification-9 report and its Lighthouse evidence remain in `.factory` and
`/work/.evidence/`. The required machine verdict is
`/work/.evidence/qa-result.json`.
