# Capacity Sentinel repair 7 handoff

## Result

**PASS — 0 unresolved product findings and 0 untested public claim families.**

Capacity Sentinel monitors synthetic model API requests for capacity errors,
latency regressions, and broken structured JSON before users report failures.
It is for teams running applications that call model APIs. The first action is
**Try it with sample data**, which opens the isolated populated `/demo` workspace.

## What changed

- Expanded `.factory/claims.json` from 12 to 25 outcome-based public claims.
  Every ID has exactly one matching `@claim:` test and every declared command
  passes from the documented setup.
- Added runner regressions for scheduled probes, real availability/latency/p95
  evidence, advertised failure classifications, alert attribution and recovery,
  edit-history preservation, caps, safety blocking, deletion, API access,
  private canary boundaries, durable restart state, non-root startup, and the
  Atlas 365-day window.
- Completed the Atlas lifecycle coverage: returned and pasted token storage,
  token-bound daily cache, invalid/revoked failure, offline retry, exact $39
  one-time copy, and the licensed comparison view.
- Fixed a license-cache safety issue: cached verdicts are now bound to the
  license token, preventing a newly returned token from reusing a prior
  token’s verdict.
- Added browser checks for service-worker update and offline demo reload.

## Durable SQLite diagnosis and deployment

The earlier `recovery-safe` restoration crash-looped before binding its port.
The active source already contains the durable correction: one SQLite
connection through `unix-dotfile`, a 30-second busy timeout, and read-only
current-schema validation before migrations. Product-only inspection before
this repair confirmed that generated revision `0000009` was already healthy
with that path, so no manual revision recovery or storage mutation was made.

Deployment used only `/opt/fleet/lib/deploy-container.sh` with
`WO_DATA_DIR=/data`. It made a declarative app PATCH, preserved existing
environment, secrets, probes, Azure Files, ingress, and one-replica bounds,
and let Azure generate `0000010`. No named revision, copy, or manual traffic
target was created.

## Identities and rollback

- Deployed implementation: `a198e827897ba90eee54c7e5903041c2c4d5e547`
- Repository handoff/test head: `510b266` — only the Atlas-window `#[cfg(test)]`
  test and claim manifest were added after the implementation, so release
  runtime remains unchanged.
- Served `/health` build: `a198e827897ba90eee54c7e5903041c2c4d5e547`
- Active image:
  `sha256:4da8170f8d3cb15cce9caeefd494c0ba5704f987e9c929bd6914a448c986bb95`
- Compatible rollback image, previously healthy `0000009`:
  `sha256:a0e31519ca5b577a152375cdfa1fe91e6e3cc1c6cf200f44f2f04f645946b4c7`

## Verification

From the documented local setup (`npm ci`):

| Command | Result |
|---|---|
| `npm test` | PASS — 1 Vitest, 24 Rust unit/integration, 3 startup-process tests |
| `npm run check` | PASS — Svelte 0 errors/warnings; Clippy warnings denied |
| `npm run build` | PASS — `dist/`; JS 27.27 KB gzip, CSS 4.97 KB gzip |
| `npm run test:e2e -- --reporter=line` | PASS — 20 desktop/mobile checks, axe, offline reload, service-worker update |
| `npm run test:claims` | PASS — all 25 manifest commands |
| `BUILD_SHA=a198e82… cargo build --locked --release` | PASS |

Fresh 1440 px desktop and 390 px phone contexts confirmed the title, job,
audience, and sample action before scrolling. Each opened the sample in one
click, showed its persistent label, attributed 429 evidence, recovered alert,
reset action, and separate real-project screen. Demo traffic made no project
API or cross-origin request; both contexts had zero console/page errors. Live
Axe on `/demo` found zero violations at both sizes.

`verify-url.sh` passed against the HTTPS homepage: HTTP 200, title, `lang=en`,
one `h1`, main landmark, image alternatives, labelled buttons, and no console
errors. ARM and HTTPS verification found:

- `provisioningState: Succeeded`, `runningStatus: Running`
- `activeRevisionsMode: Single`; only generated
  `sf-model-capacity-sentinel--0000010` active, healthy, provisioned, and at
  one running replica
- single-mode ingress `latestRevision: 100`
- Azure Files `sf-model-capacity-sentinel-data` mounted at `/data`; min/max
  replicas remain 1
- `/` and `/health` return 200; health reports the deployed SHA above
- a 220-request invalid-access burst returned 155 expected 401s and 65 429s;
  all 65 429 responses contained `Retry-After`; two seconds later a request
  correctly returned 401 again

Evidence is in `/work/.evidence/repair-7-live/`, including verifier JSON and
desktop/phone screenshots. The required plain catalog description is copied to
`/work/.evidence/catalog-description.txt`; public Atlas billing metadata is at
`/work/.evidence/billing-offer.json`.

## Earlier finding disposition

| Finding | Current disposition |
|---|---|
| Review 1 F1 demo sandbox | Fixed and checked locally and live. |
| Review 1 F2 / Review 2 F1 claims | Fixed: 25 declared, complete, repeatable commands pass. |
| Review 1 F3 inert edit | Fixed; keyboard edit and history-preservation regression pass. |
| Review 1 F4 / Verification 4 limits | Fixed; local and live invalid requests receive 429 with `Retry-After`. |
| Review 1 F5 billing checkout | Checkout stays unavailable until registration; paid Atlas deliverable, exact price, restore, verification, and terms remain. |
| Review 1 F6 first-screen copy | Fixed and reconfirmed on fresh desktop and phone. |
| Review 1 F7 routes and metadata | Fixed; verifier, legal routes, titles, and designed 404 remain healthy. |
| Review 1 F8 Docker base | Fixed: `rust:1-slim` builder and distroless non-root runtime remain. |
| Review 1 F9 phone targets | Fixed; mobile/keyboard/axe regression passes. |
| Verification 6 failed restoration | Superseded by durable SQLite and healthy declarative `0000009` then `0000010`. |
| Verification 7 identities | Implementation, documentation/test, and served SHAs are recorded separately. |

## Known dependency

Atlas billing registration remains pending with the separate factory billing
operator. The product deliberately does not show a broken checkout link. A
valid future Sociobot license can be restored and verified, but entitlement
cannot be issued until registration exists. The free single-project runner and
all safety/export behavior remain available without it.
