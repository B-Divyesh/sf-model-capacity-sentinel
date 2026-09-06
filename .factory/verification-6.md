# Verify model API capacity monitoring — FAIL

Verified 2026-09-06 UTC against implementation candidate
`90109876b120284fa7390a7342774a37c605b452`, documentation commit
`3b0372c63895a5c62b7a55cad9f6d82f29473d01`, and
<https://model-capacity-sentinel.sociobot.in>.

## Verdict

**FAIL.** The candidate passes all declared local checks and every declared
claim command, but the deployed product is unavailable. Fresh desktop and
phone browser visits cannot load the first screen, and both public `/` and
`/health` time out. A user therefore cannot use the one-click sample or the
real product.

Finding count: **1**. Untested public claim count: **0**.

## Job, audience, and first action

- Job: monitor synthetic model API requests for capacity errors, slower
  responses, and broken JSON output.
- Audience: teams running applications that call one or more model APIs.
- First action in the reviewed candidate: **Try it with sample data**. It
  opens the populated `/demo` workspace.

The live service never returned a document, so these are verified from the
clean candidate and not observable on the public site.

## Finding

### F1 — Critical — The public product does not respond

At 2026-09-06 UTC, fresh Chromium contexts at 1440 × 900 and iPhone 13 size
each timed out after 12 seconds while navigating to the HTTPS homepage. The
same result occurred outside the browser:

```text
GET /        HTTP 000 after 15.002711s
GET /health  HTTP 000 after 15.002935s
```

This confirms the redacted isolation evidence rather than replacing it: the
sole `recovery-safe` revision is crash-looping before it opens its port. The
configured implementation SHA is `90109876b120284fa7390a7342774a37c605b452`,
but no response proves it is served. The rollback-candidate digest remains
unactivated. No ARM settings, revisions, secrets, or product data were
changed during this verification.

Required next action: repair and deploy the durable SQLite startup condition
while preserving the single-revision, one-replica `/data` topology. Then
repeat fresh desktop and phone visits, `/health`, and the live rate-limit
check. Do not treat a local build as a restoration of the public product.

## Clean-checkout evidence

A new detached clone of `3b0372c63895a5c62b7a55cad9f6d82f29473d01` was used.
The clone was clean before installation. The implementation candidate differs
from that documentation commit only in `.factory` reports and `README.md`;
the last product-code commit is `9010987`.

| Command | Result |
|---|---|
| `npm ci` | PASS — 134 packages; 0 reported vulnerabilities |
| `npm test` | PASS — 2 Vitest tests and 10 Rust tests plus the startup-process test |
| `npm run check` | PASS — Svelte check and `cargo clippy -D warnings` |
| `npm run build` | PASS — produced `dist/` |
| `npm run test:e2e -- --reporter=line` | PASS — all 18 desktop/mobile claim tests |
| `npm run test:claims` | PASS — executed every manifest command below |
| `BUILD_SHA=90109876b120284fa7390a7342774a37c605b452 cargo build --locked --release` | PASS |

The declared claims all passed from the clean sample or temporary-runner
sandbox. No claim command was missing or failed.

| Claim | Declared command | Result |
|---|---|---|
| `demo-sandbox` | Playwright `@claim:demo-sandbox` | PASS |
| `demo-reset` | Playwright `@claim:demo-reset` | PASS |
| `sample-monitoring-output` | Playwright `@claim:sample-monitoring-output` | PASS |
| `edit-probe` | Playwright `@claim:edit-probe` | PASS |
| `csv-export` | Playwright `@claim:csv-export` | PASS |
| `accessible-mobile-dashboard` | Playwright `@claim:accessible-mobile-dashboard` | PASS |
| `route-structure` | Playwright `@claim:route-structure` | PASS |
| `access-code-rate-limit` | Playwright `@claim:access-code-rate-limit` | PASS |
| `encrypted-storage-and-classification` | `cargo test claim_encrypted_storage_and_failure_attribution` | PASS |
| `public-endpoint-safety` | `cargo test claim_public_endpoint_safety` | PASS |
| `local-persistence-health` | `cargo test claim_local_persistence_health` | PASS |
| `license-restore` | `npx vitest run -t @claim:license-restore` | PASS |

## Local runtime checks

An isolated release binary was started with a temporary SQLite directory and
a locally supplied test access code. No production data or credential was
read. It returned `200` from `/health`, returned `401` for invalid access,
rejected a loopback probe target with `400`, and after a fixed-client
invalid-access burst returned 2 `429` responses; both had `Retry-After`.
Stopping and restarting the same binary with the same data directory again
returned `200` from `/health`.

The passing browser suite verifies the populated demo, its persistent sample
label, reset, real-project separation, CSV output, keyboard operation,
phone-sized layout, focus, reduced-motion stylesheet behavior, route titles,
privacy and terms, and the designed deliberate 404. It uses Playwright's axe
integration and found no serious or critical violations in the tested states.
There is no repository `verify-url.sh`; the equivalent page-structure and axe
checks are part of the passing browser suite. The local demo records no
`/api/*` request and only same-origin requests.

## Earlier findings

| Earlier finding | Current disposition |
|---|---|
| Review 1 F1 sample sandbox | Locally fixed; demo and isolation claim pass. Not live-verifiable because F1 above prevents loading `/demo`. |
| Review 1 F2 missing claims | Fixed; manifest contains 12 entries and all 12 declared commands pass. |
| Review 1 F3 inert edit action | Fixed locally; keyboard edit-and-save claim passes. |
| Review 1 F4 public rate limit | Fixed locally; invalid accesses receive 429 with `Retry-After`. Live allowance cannot be checked while the service is down. |
| Review 1 F5 broken paid checkout | Copy now states billing registration is pending and does not offer checkout; license restoration has a passing local test. |
| Review 1 F6 first-screen copy | Fixed in candidate copy: job, audience, and sample action are present before scrolling. |
| Review 1 F7 routes and metadata | Fixed locally by the route-structure claim, including a deliberate designed 404. Live route checks are blocked by F1. |
| Review 1 F8 Rust Docker base | Fixed in source: `rust:1-slim`. Docker is unavailable in this verifier, so image construction was not independently run. |
| Review 1 F9 small phone targets | Fixed locally; the mobile claim asserts all header/footer links are at least 44 × 44 px. |
| Verification 1 security, plaintext, and populated-axe findings | Fixed by the passing encrypted-storage, public-endpoint, and populated browser axe coverage. |
| Verification 3 backend build identity | Source and local `/health` use the candidate SHA. Public identity is not observable while F1 persists. |
| Verification 4 read limits and missing retry header | Fixed locally by claim and runtime evidence. |
| Verification 5 PASS evidence | Superseded for live acceptance by F1: its previously healthy runtime is no longer reachable. |

## Scope and evidence classification

The implementation SHA is `90109876b120284fa7390a7342774a37c605b452`.
The documentation SHA is `3b0372c63895a5c62b7a55cad9f6d82f29473d01`.
The latter is report-only relative to the former. The redacted restoration
state in `.factory/isolation-2026-09-05.md` was reviewed without reading any
secret values. Local test evidence establishes candidate behavior; the live
timeouts establish the deployment defect. They must not be conflated.
