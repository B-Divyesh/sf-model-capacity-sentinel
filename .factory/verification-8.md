# Verify model API capacity monitoring — verification 8

Date: 2026-09-06  
Live URL: <https://model-capacity-sentinel.sociobot.in>

## Verdict

**FAIL — 3 findings and 2 untested public claims.**

The current product is healthy and its core monitoring, sample, privacy,
accessibility, rate-limit, persistence, and deployment checks pass. It cannot
receive a PASS because the contract requires zero findings of every severity
and zero untested public claims.

No product code, deployment setting, revision, traffic rule, storage, secret,
or product data was changed during this verification.

## Job, audience, and first action before scrolling

- Job: monitor model API capacity, latency, and structured JSON output early.
- Audience: teams running applications that call model APIs.
- First action: **Try it with sample data**.

Fresh 1440 × 900 desktop and 390 × 844 phone contexts showed all three before
scrolling. The action opened the populated `/demo` workspace in one click.

## Findings

### F1 — High — Two public promises remain outside complete claim coverage

All 26 entries in `.factory/claims.json` have one matching tag and every
declared command passes. The public-copy cross-check still found two promises
that are absent from the manifest and are not completely asserted by another
claim test:

1. The dashboard says its availability figure uses the **“Last 20 observations
   per probe.”** The `real-monitoring-metrics` test creates three observations;
   it does not create more than 20 and prove that older observations are
   excluded from the availability and p95 calculation.
2. The access screen says the project access code **“stays in this browser
   session.”** No declared test proves that the code is stored only in session
   storage, is absent from persistent browser storage, and is gone in a fresh
   browser session.

Source inspection supports both statements: the metric query has `LIMIT 20`
and the access code uses `sessionStorage`. The claims contract requires an
observable tagged sandbox test, so source inspection is not a substitute.

Required repair: list both promises in `.factory/claims.json` and add one
complete tagged test for each, or remove/narrow the public copy.

### F2 — Medium — The required route skeleton is incomplete

The normal application footer contains the product line, Privacy, Terms,
Source, and the build SHA, but it does not contain the required **“Built by
Param Factory”** attribution on any route.

The designed 404 returns the correct HTTP 404 and has a title, `h1`, `main`,
product styling, and a way home. It does not have the required skip link. Its
header also omits the normal Dashboard link, and its footer omits Privacy, the
build identifier, and the Param Factory attribution. This does not meet the
contract that every route uses a consistent header and footer and provides a
skip link to `#main`.

Required repair: complete the normal footer attribution and bring the 404
header, skip link, and footer into the same required route skeleton.

### F3 — Low — The privacy email touch target is 20 px high on a phone

At 390 × 844, `privacy@sociobot.in` measures 172 × 20 CSS px. Every other
visible interactive target on `/`, `/demo`, `/privacy`, and `/terms` met the
44 × 44 px baseline. This means the earlier small-target finding is mostly,
but not completely, fixed.

Required repair: enlarge the email link's interactive area to at least 44 px
high without changing the text size.

## Candidate and live provenance

| Identity | Value | Result |
|---|---|---|
| Implementation candidate reviewed | `0701cd3b506432c10c21ec9fa5b1cfd82a54c2cf` | PASS |
| Last production-behavior change | `a198e827897ba90eee54c7e5903041c2c4d5e547` | Recorded |
| Documentation and repository HEAD | `3a8a37f492eaa3dc83d72b81d94afb7a3c42a15d` | PASS |
| Live `/health` build | `3a8a37f492eaa3dc83d72b81d94afb7a3c42a15d` | PASS |
| Active immutable image | `sha256:ecf3333dd7585b8dff4b2227ab2aa8a9f1749748f2678e51cfd85fdca55b5282` | PASS |

`git diff 0701cd3..3a8a37f` changes only `.factory/handoff.md`. A production
frontend rebuilt with build SHA `3a8a37f…` had byte-identical JavaScript and
CSS to the live assets. The later report-only commit therefore does not change
the product implementation represented by candidate `0701cd3`.

## Clean-checkout verification

Fresh detached checkout:
`/tmp/mcs-v8-clean.GjJ8NE` at documentation SHA `3a8a37f…`.

| Command | Result |
|---|---|
| `npm ci` | PASS — 134 packages; 0 reported vulnerabilities |
| `npm audit --omit=dev` | PASS — 0 vulnerabilities |
| `npm test` | PASS — 1 Vitest, 24 Rust unit/integration, and 3 process tests |
| `npm run check` | PASS — Svelte 0 errors/warnings; Clippy warnings denied |
| `npm run build` | PASS — `dist/` produced |
| `npm run test:e2e -- --reporter=line` | PASS — 22 desktop/mobile tests |
| `npm run test:claims` | PASS — all 26 declared commands and tag audit |
| `BUILD_SHA=3a8a37f… cargo build --locked --release` | PASS |

Built frontend sizes were 75.19 KB JavaScript and 18.54 KB CSS (27.27 KB and
4.97 KB gzip). The hero is 129,198 bytes, the social image is 1200 × 630, and
the Apple touch icon is 180 × 180.

A separate release process using a temporary SQLite directory returned its
build SHA from `/health`. A disabled valid probe at documented maximum/minimum
boundaries returned 201, interval zero returned 400, a loopback endpoint
returned 400, an invalid access code returned 401, and the authenticated
summary returned the saved probe. The declared restart-persistence and
non-root process claims also passed.

## Declared claim results

Every declared command was run from the fresh checkout. All passed.

| Claim ID | Result |
|---|---|
| `demo-sandbox` | PASS |
| `demo-reset` | PASS |
| `browser-privacy-egress` | PASS |
| `sample-monitoring-output` | PASS |
| `edit-probe` | PASS |
| `csv-export` | PASS |
| `accessible-mobile-dashboard` | PASS |
| `route-structure` | PASS |
| `access-code-rate-limit` | PASS |
| `encrypted-storage-and-classification` | PASS |
| `public-endpoint-safety` | PASS |
| `local-persistence-health` | PASS |
| `atlas-license-lifecycle` | PASS |
| `scheduled-probes` | PASS |
| `real-monitoring-metrics` | PASS |
| `failure-classification` | PASS |
| `attributed-alert-recovery` | PASS |
| `edit-preserves-history` | PASS |
| `token-limits` | PASS |
| `probe-deletion` | PASS |
| `api-access-coverage` | PASS |
| `private-canary-boundary` | PASS |
| `durable-project-state` | PASS |
| `nonroot-runner` | PASS |
| `atlas-comparison-view` | PASS |
| `atlas-365day-window` | PASS |

The two unlisted promises in F1 set `untested_claim_count` to **2** despite all
declared commands passing.

## Live product evidence

- The final independent Playwright suite passed 10/10 checks across fresh
  desktop and phone contexts. The one-click sample showed two realistic probes,
  attributed 429 evidence, an open alert, and a recovered alert.
- Editing the sample changed only
  `demo:capacity-sentinel:summary`. The persistent sample label remained
  visible, Reset demo restored the shipped records, and Start for real opened
  the separate access-code state. No project API or cross-origin request was
  made during this flow.
- The factory URL verifier passed `/` and `/demo`: HTTPS 200, correct title,
  `lang=en`, one `h1`, one `main`, complete image alternatives, labelled
  buttons, and no console error.
- Full Axe scans returned zero violations on `/`, `/demo`, `/privacy`,
  `/terms`, and the expected 404. Manual inspection found the skip-link issue
  in F2, which Axe does not flag.
- Keyboard focus, dialog first focus, reduced motion, offline demo reload,
  service-worker update, 200% phone text, and no horizontal overflow passed.
- `/`, `/demo`, `/privacy`, `/terms`, `robots.txt`, `sitemap.xml`, favicon,
  touch icon, and social image returned 200. The unknown route deliberately
  returned 404 with its designed body. The rendered GitHub source link returned
  200; the privacy mail link is the target-size finding in F3.
- Security headers include CSP, frame denial, no-sniff, and
  `Referrer-Policy: no-referrer`. Hashed assets use one-year immutable caching.
- A 90-request invalid-access burst returned 43 × 401 and 47 × 429. Every 429
  had `Retry-After: 1`; after 2.1 seconds the same client received 401. Health
  remained exempt and returned 200.
- Lighthouse mobile scored 99 performance, 100 accessibility, 100 best
  practices, and 100 SEO. LCP was 1.797 s, total blocking time 89 ms, CLS 0,
  and total transfer 162,649 bytes.

No provider credential was available or invented. Real provider outcomes were
verified through the repository's local endpoint fixtures, not by spending
against a third-party model API.

## Azure deployment and rollback record

Only `sf-model-capacity-sentinel` Container App, revision, replica, and
`sf-model-capacity-sentinel` image metadata were read. No deployment state was
changed.

| Check | Current evidence |
|---|---|
| ARM state | `provisioningState: Succeeded`; `runningStatus: Running` |
| Revision mode | `Single` |
| Active revision | Exactly one: generated `sf-model-capacity-sentinel--0000013` |
| Active health | `Healthy`, `Provisioned`, `RunningAtMaxScale` |
| Replica | Exactly one running/ready replica; restart count 0 |
| Traffic | `latestRevision: true`, weight 100 |
| Scale | `minReplicas: 1`, `maxReplicas: 1` |
| Durable storage | Azure Files volume `sf-model-capacity-sentinel-data` mounted at `/data` |
| Public health | HTTPS 200 with `status: ok` and build `3a8a37f…` |
| Active image | `sha256:ecf3333dd7585b8dff4b2227ab2aa8a9f1749748f2678e51cfd85fdca55b5282`, tag `3a8a37f492ea` |

Rollback evidence is intact:

- Revision `0000012` is a healthy, provisioned, stopped zero-traffic revision
  for candidate `0701cd3`, using digest
  `sha256:75b3f63e0cced5ea1a068eeeb7d90a300901442aba4dc914a6c413bd393e03b1`.
- Revision `0000011` is a healthy, provisioned, stopped zero-traffic revision
  using digest
  `sha256:20f9aaffd1423cb6ac2f6a4087c95a60cfa06b0fef3b13b0619d4cc3d1d9006f`.
- Both immutable digests remain present in the product's registry repository.

The healthy Single-mode, one-replica `/data` topology and latest-revision
traffic were left unchanged.

## Earlier finding disposition

| Earlier finding | Current disposition |
|---|---|
| Review 1 F1: missing sample | Fixed; fresh local and live one-click sample, label, realistic data, reset, and separation pass. |
| Review 1 F2 / Review 2 F1: missing or incomplete claims | Mostly fixed; 26 declared commands pass, but F1 finds two remaining untested public promises. |
| Review 1 F3: inert edit | Fixed; keyboard and pointer edit pass, and backend history preservation passes. |
| Review 1 F4 / Verification 4: rate limiting | Fixed; local and live 429, `Retry-After`, recovery, and health exemption pass. |
| Review 1 F5: broken checkout | Fixed by withholding checkout while registration is pending; restore and invalid/revoked handling pass. |
| Review 1 F6: unclear first screen | Fixed on desktop and phone. |
| Review 1 F7: incomplete routes and metadata | Mostly fixed; titles, metadata, assets, legal routes, and designed 404 pass, but F2 records remaining skeleton omissions. |
| Review 1 F8: pinned Rust base | Fixed; Dockerfile uses `rust:1-slim` and the locked native release build passes. |
| Review 1 F9: small phone targets | Mostly fixed; F3 records the remaining 20 px-high privacy email target. |
| Verification 1: public writes, plaintext, populated Axe, and caching | Fixed; access coverage, encrypted storage, egress/privacy tests, Axe, and immutable asset headers pass. |
| Verification 3: build identity, console, and startup record | Fixed; live identity, clean consoles, and local structured startup record pass. |
| Verification 6: failed restoration and public outage | Historical; current health, ARM state, replica, storage, and traffic all pass. |
| Verification 7 PASS | Runtime and infrastructure remain healthy; its claim and minor-structure conclusions are superseded by F1–F3. |
| Repair8 builder PASS | Declared tests and deployment are confirmed, but independent copy and route-contract review supersedes its zero-finding assertion. |

## Evidence

Command logs, browser screenshots, request results, Lighthouse output, asset
hashes, and scoped Azure JSON are under `/work/.evidence/verification-8/`.
The required report copy is `/work/.evidence/qa-report.md` and the machine
result is `/work/.evidence/qa-result.json`.
