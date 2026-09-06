# Verify model API capacity monitoring — PASS

Verified 2026-09-06 UTC from a clean checkout of implementation
`700c520391446b89b7ca82570ccb5e9cb7cbf716`. The documentation commit served
at verification was `784a4d372874016fcdce97271a18c7d79afcbd0a`; it changes
only this handoff and verification report, not product code. The live service
returns that documentation build SHA, so its served source is the reviewed
implementation plus report-only changes.

## Verdict

**PASS — 0 findings and 0 untested public claims.**

## Job, audience, and first action

- Job: monitor model API capacity, latency, and structured JSON output early.
- Audience: teams running applications that call one or more model APIs.
- First action: **Try it with sample data**.

Before scrolling, fresh desktop and 390 px phone browsers showed the job,
audience, primary sample action, and its result. The sample opened realistic
providers, a 429 capacity alert, and a recovered latency alert. The persistent
**Demo — sample data, nothing is saved** label, **Reset demo**, and **Start for
real** all worked. The real-project access screen remained separate. Neither
session logged a console or page error and neither overflowed horizontally.

## Clean checkout and claims

Clean checkout: `/tmp/model-capacity-sentinel-verify-7.fwJSU6` at `700c520`.

| Command | Result |
|---|---|
| `npm ci` | PASS, 0 vulnerabilities |
| `npm test` | PASS: 2 Vitest, 14 Rust unit/integration, 1 startup test |
| `npm run check` | PASS: Svelte check and Clippy with warnings denied |
| `npm run build` | PASS; `dist/` produced (27.17 KB gzipped initial JS, 4.97 KB gzipped CSS) |
| `npm run test:e2e -- --reporter=line` | PASS: 16 desktop/mobile tests |
| `npm run test:claims` | PASS: every declared command below |
| `BUILD_SHA=700c520391446b89b7ca82570ccb5e9cb7cbf716 cargo build --locked --release` | PASS |

All twelve `.factory/claims.json` entries passed their declared commands:
`demo-sandbox`, `demo-reset`, `sample-monitoring-output`, `edit-probe`,
`csv-export`, `accessible-mobile-dashboard`, `route-structure`,
`access-code-rate-limit`, `encrypted-storage-and-classification`,
`public-endpoint-safety`, `local-persistence-health`, and `license-restore`.
There are no missing, failed, incomplete, or untested declared claims.

The local tests cover normal operations, invalid access, public-endpoint
rejection, rate-limit recovery guidance, encrypted storage, persistence across
restart, demo isolation, keyboard operation, reduced motion, phone layout,
legal routes, and the deliberate designed 404. The live `verify-url.sh` checks
for `/` and `/demo` passed: HTTP 200, title, `lang=en`, exactly one `h1`, a
main landmark, image alt text, labelled buttons, and zero console errors. Live
axe scans of `/demo` in fresh desktop and phone contexts had zero violations,
including zero serious or critical violations.

## Live Azure verification

The product-only Container App inspection and HTTPS checks found:

| Check | Evidence | Result |
|---|---|---|
| ARM state | `provisioningState: Succeeded`, `runningStatus: Running` | PASS |
| Revision mode | `activeRevisionsMode: Single` | PASS |
| Active revision | only `sf-model-capacity-sentinel--0000009` is active; `Healthy`, `Provisioned`, `RunningAtMaxScale`, one replica | PASS |
| Traffic | `latestRevision: true`, weight `100` | PASS |
| Durable topology | `minReplicas: 1`, `maxReplicas: 1`; Azure Files volume `sf-model-capacity-sentinel-data` mounted at `/data` | PASS |
| Image | `sociobotregistry.azurecr.io/sf-model-capacity-sentinel@sha256:a0e31519ca5b577a152375cdfa1fe91e6e3cc1c6cf200f44f2f04f645946b4c7` | PASS |
| Homepage | `HTTPS /` returned 200 | PASS |
| Health | `HTTPS /health` returned 200 with build `784a4d372874016fcdce97271a18c7d79afcbd0a` | PASS |
| Public rate limit | 220 invalid accesses: 171 expected 401 and 49 429; every 429 had `Retry-After: 1` | PASS |
| Routes | `/demo`, `/privacy`, `/terms`, `robots.txt`, `sitemap.xml`, and favicon returned 200; an unknown route returned the expected designed 404 | PASS |

The live health SHA is the current report-only documentation SHA. `git diff
700c520..784a4d3` contains only `.factory/handoff.md` and
`.factory/verification-7.md`; the product source checksum is unchanged. This
is therefore a valid match to the last implementation candidate, with both
SHAs recorded rather than conflated.

## Revision history and rollback

Exactly five prior manually named revisions remain inactive, stopped, and at
zero traffic: `sqlite-retry`, `sqlite-safe`, `schema-safe`, `schema-diag`, and
`recovery-safe`. No named revision was created after `recovery-safe`; the
current `0000009` revision has Azure's generated name and is the only active
revision. No named traffic target is configured.

The documented rollback digest is
`sha256:6e28640d4bb7d4118f2542cd67c4ae06d883ac26e940a53c0d163f796d16d1ed`,
the previous healthy declarative recovery image. The current active digest is
recorded above. Both use the compatible SQLite locking implementation; the
older default-lock digests must not be used on the retained Azure Files volume.

## Earlier findings

| Earlier finding | Current disposition |
|---|---|
| Review 1 F1: no sample sandbox | Fixed; clean and live demo checks pass. |
| Review 1 F2: absent claims/tests | Fixed; 12 declared commands pass. |
| Review 1 F3: edit control inert | Fixed; keyboard edit claim passes. |
| Review 1 F4 / Verification 4: public limit missing or no retry header | Fixed; live overload returns 429 with `Retry-After: 1`. |
| Review 1 F5: broken checkout offer | Fixed by removing the unavailable checkout offer; restore path is tested. |
| Review 1 F6: missing first-screen job/audience/action | Fixed; confirmed before scrolling on both viewports. |
| Review 1 F7: routes/metadata incomplete | Fixed; titles, legal routes, structure, and deliberate 404 pass. |
| Review 1 F8: pinned Rust base | Fixed in source (`rust:1-slim`); release build passes. |
| Review 1 F9: small phone targets | Fixed by mobile claim; phone browser has no overflow. |
| Verification 1: public write, plaintext, and populated axe issues | Fixed by access, encryption/classification, and axe claim coverage. |
| Verification 3: identity, console, and startup-record issues | Fixed; live health has a build SHA and browser checks have no errors. |
| Verification 5 PASS | Reconfirmed after the subsequent failed restoration. |
| Verification 6 F1: public outage / failed recovery | Fixed; ARM, revision health, homepage, and health all pass. |

## Evidence classification

Local command logs are under `/tmp/mcs-*-7.log`; live screenshots and
`verify-url.sh` output are under `/work/.evidence/`. This verification read
only this product's Container App, revisions, and public URL. It did not read
or change product data, settings, secrets, traffic, revisions, or any other
service.
