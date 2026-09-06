# Capacity Sentinel repair 8 handoff

## Result

**PASS — 0 unresolved product findings and 0 untested public claim families.**

Capacity Sentinel monitors synthetic model API requests for capacity errors,
latency regressions, and broken structured JSON. It is for teams that operate
applications using model APIs. The first action is **Try it with sample data**,
which opens the isolated populated `/demo` workspace in one click.

## What changed

- The claim runner now fails unless every manifest ID has exactly one matching
  `@claim:` test and every tagged test is listed in the manifest.
- Added an outcome-based browser privacy claim. It records every request while
  opening the product and demo, and rejects analytics, third-party resources,
  cross-origin traffic, and project API calls.
- Strengthened target safety through the real create route. The claim now
  rejects unspecified, IPv4 and IPv6 loopback, private, carrier-grade NAT,
  unique-local, and link-local addresses, and proves none are stored.
- Strengthened JSON-path coverage with successful nested-path validation and
  a missing nested-path failure, alongside invalid JSON, timeout, network, and
  upstream classifications.
- Strengthened private-canary evidence by inspecting API output and durable
  SQLite/WAL files for the request credential, synthetic prompt, and provider
  response content.
- Corrected the non-root test, which had checked the process ID instead of the
  user ID. It now launches the runner as UID 65532 and verifies that user owns
  the created durable database.
- Expanded Atlas fixtures to cover valid, invalid, revoked, offline, cached,
  and older-than-one-day verdicts. Returned and pasted tokens, token-bound
  caching, the licensed comparison, exact $39 one-time copy, and the 365-day
  window remain covered.

The manifest now contains 26 one-to-one tagged claims. The 12 families from
strict review 2 have complete outcome tests: scheduling; real metrics; JSON
paths and classifications; alert attribution/recovery; edit history; token
caps; target safety; deletion; API access; privacy/egress; durable non-root
SQLite; and Atlas.

## Durable SQLite diagnosis

The failed `recovery-safe` restoration is historical. Before this repair, the
public app was already healthy on generated revision `0000011`. The durable
fix remains one SQLite connection using the `unix-dotfile` VFS, a 30-second
busy timeout, and read-only validation when the schema is current.

The new revision's actual startup log against the existing `/data` mount shows
two applied and two expected migrations with `matches=true`, followed by
`database schema is current`. It selected the existing recovery database,
reused persisted key and access-code material, opened port 8080, and has zero
container restarts. No product data, credential, or secret value was read.

## Deployment and identities

Deployment used only `/opt/fleet/lib/deploy-container.sh` with
`WO_DATA_DIR=/data`. The wrapper issued one declarative app PATCH, preserved
the environment, secrets, ingress, Azure Files volume, and one-replica bounds,
and let Azure generate revision `sf-model-capacity-sentinel--0000012`. No
revision was named, copied, activated, or manually assigned traffic.

- Last runtime-behavior implementation: `a198e827897ba90eee54c7e5903041c2c4d5e547`
- Tested, pushed, deployed, and served candidate: `0701cd3b506432c10c21ec9fa5b1cfd82a54c2cf`
- Candidate image: `sha256:75b3f63e0cced5ea1a068eeeb7d90a300901442aba4dc914a6c413bd393e03b1`
- Compatible rollback image, previously healthy revision `0000011`:
  `sha256:20f9aaffd1423cb6ac2f6a4087c95a60cfa06b0fef3b13b0619d4cc3d1d9006f`
- This handoff is a later documentation-only commit and is not a reason to
  rebuild the product image.

Final product-only Azure state:

- `provisioningState: Succeeded`; `runningStatus: Running`
- `activeRevisionsMode: Single`
- exactly one active revision: generated `0000012`, Healthy and Provisioned
- exactly one running replica with zero restarts
- ingress traffic is `latestRevision: true`, weight `100`
- `minReplicas: 1`, `maxReplicas: 1`
- existing `sf-model-capacity-sentinel-data` Azure Files volume at `/data`
- existing environment, secrets, ingress, and absence of explicit probes
  preserved
- `/` and `/health` return 200; health serves candidate `0701cd3b…`

## Verification

The documented setup was run from clean clone
`/tmp/model-capacity-sentinel-repair-8-clean.PozTVZ` at the served candidate.

| Command | Result |
|---|---|
| `npm ci` | PASS — 0 reported vulnerabilities |
| `npm test` | PASS — 1 Vitest, 24 Rust unit/integration, 3 process tests |
| `npm run check` | PASS — Svelte 0 errors/warnings; Clippy warnings denied |
| `npm run build` | PASS — `dist/`; JS 27.27 KB gzip, CSS 4.97 KB gzip |
| `npm run test:e2e -- --reporter=line` | PASS — 22 desktop/mobile checks |
| `npm run test:claims` | PASS — all 26 manifest commands and tag audit |
| `BUILD_SHA=0701cd3… cargo build --locked --release` | PASS; health returned that SHA |

Fresh live 1440 × 900 desktop and 390 × 844 phone contexts confirmed the job,
audience, and first action before scrolling. Each opened the populated sample,
showed the persistent sample label, changed and reset sample data, and returned
to a separate real-project access screen. Both made zero project API and zero
cross-origin requests, logged zero console/page errors, had no horizontal
overflow, and had zero serious or critical Axe findings.

The factory URL verifier passed `/` and `/demo`: HTTP 200, title, `lang=en`, one
`h1`, `main`, image alternatives, labelled buttons, and no console errors.
Privacy and Terms returned 200 with distinct titles; the deliberate unknown
route returned a designed 404 with one `h1`, `main`, and a home link. Every
rendered link returned 200–399 or was the documented privacy email link.
Service-worker update and an offline demo reload passed without errors.

A live 220-request invalid-access burst returned 46 expected 401 responses and
174 rate-limited responses. Every 429 had a positive `Retry-After`; after 2.1
seconds the same client received 401 again. All 60 concurrent health requests
returned 200.

Lighthouse mobile: performance 100, accessibility 100, best practices 100,
SEO 100; LCP 1.755 s, total blocking time 47 ms, CLS 0, and transfer 162,639
bytes. Evidence and screenshots are under `/work/.evidence/repair-8-live/`.
The catalog description is copied to
`/work/.evidence/catalog-description.txt`. Public offer metadata is at
`/work/.evidence/billing-offer.json`.

## Earlier finding disposition

| Finding | Current disposition |
|---|---|
| Review 1: missing demo | Fixed; clean and live label, populated sample, reset, and isolation pass. |
| Review 1 / Review 2: incomplete claims | Fixed; 26 manifest entries, unique-tag audit, and every command pass. |
| Review 1: inert edit | Fixed; keyboard/pointer edit and backend history preservation pass. |
| Review 1 / Verification 4: rate limiting | Fixed; local and live 429 plus `Retry-After` and recovery pass. |
| Review 1: broken checkout | Checkout stays unavailable pending registration; the paid deliverable, exact price, restore, verification, and legal copy remain. |
| Review 1: unclear first screen | Fixed and reconfirmed before scrolling on desktop and phone. |
| Review 1: routes and metadata | Fixed; route titles, links, legal pages, and designed 404 pass. |
| Review 1: pinned Rust base | Fixed; `rust:1-slim` and locked release build pass. |
| Review 1: phone targets | Fixed; mobile browser and accessibility checks pass. |
| Verification 1: public writes, plaintext, populated Axe | Fixed; access, encryption/raw-storage privacy, SSRF, and populated Axe checks pass. |
| Verification 3: identity, console, startup record | Fixed; served SHA, clean consoles, and non-secret startup record pass. |
| Verification 6: crash-looping restoration | Superseded; existing `/data` schema is current and generated revision `0000012` is healthy with zero restarts. |
| Strict review 2: 12 claim families | Fixed with complete tagged outcome tests listed above. |

## Known dependency

Atlas billing registration remains pending with the separate factory billing
operator. The product truthfully withholds checkout until registration. A
valid future Sociobot license can be restored and verified. The free
single-project runner, safety controls, alerts, editing, and CSV export work
without Atlas. No external provider credential was available or invented;
provider behavior is verified with local recorded fixtures.
