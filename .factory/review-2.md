# Review 2: Monitor model API capacity and output shape

- Review date: 2026-09-06 UTC
- Live URL: <https://model-capacity-sentinel.sociobot.in>
- Implementation candidate: `700c520391446b89b7ca82570ccb5e9cb7cbf716`
- Repository documentation head: `5ef195d1d40316ef1048612d3de2c93c20e9afe2`
Served build: `784a4d372874016fcdce97271a18c7d79afcbd0a`

## Verdict

**FAIL — 1 finding and 12 untested public claim families.**

The deployed product is healthy and its main flows work. The release still
fails the strict claims contract. The 12 entries already present in
`.factory/claims.json` all pass, but the public site and README make additional
promises that have no matching claim entry or whose tagged test does not assert
the complete promise.

## Job, audience, and first action before scrolling

- Job: monitor model API capacity, latency, and structured JSON output early.
- Audience: teams running applications that call model APIs.
- First action: **Try it with sample data**.

Fresh 1440 × 900 desktop and 390 × 844 phone contexts showed all three before
scrolling. The action opened `/demo` in one click.

## Finding

### F1 — High — The claim inventory and tagged tests remain incomplete

All 12 declared claim commands pass, and every manifest ID has exactly one
`@claim:` tag. That does not cover all public promises. The following 12 claim
families are absent from the manifest or are only partly asserted by the named
test:

1. Scheduled execution of enabled probes.
2. Availability, elapsed latency, and rolling p95 derived from real successful
   endpoint responses rather than shipped demo records.
3. Required JSON-path validation and the advertised invalid-JSON, invariant,
   timeout, network, and upstream failure classifications.
4. Provider/model attribution and automatic recovery of real backend alerts.
5. Preservation of observations when a probe is edited. The tagged edit test
   changes the model but never asserts that history remains.
6. Enforcement of the daily token cap and maximum output-token setting.
7. Rejection of every advertised private, loopback, and link-local target. The
   tagged safety test exercises only `127.0.0.1`.
8. Deletion of the probe's settings, observations, and alerts. The backend test
   checks only the `204` response.
9. Access-code protection across every `/api/*` route. The tagged test exercises
   only `/api/summary`.
10. The privacy and egress promises: no analytics, no retained response content,
    and sending the synthetic prompt and credential only to the configured
    endpoint.
11. Durable SQLite project-state behavior and the documented non-root `/data`
    container contract. The tagged persistence test checks generated key files,
    not project data or the container user.
12. The full Atlas promise. The tagged license command tests URL-token storage;
    it skips the pasted-token test and does not fixture-test verification,
    daily caching, invalid/revoked handling, or the 365-day comparison view.

Several of these behaviors were observed manually or are visible in source,
including restart persistence, invalid-token handling, same-origin demo traffic,
and the 365-day query. The claims contract still requires each retained public
claim to be listed with one complete tagged sandbox test. Manual review is not a
substitute for that repeatable evidence.

Required repair: inventory the retained public claims, add one complete tagged
test per claim family, and remove or narrow copy that cannot be proved. In
particular, make each test assert the result promised rather than only a status
code, control, or canned record.

## Clean-checkout verification

Fresh checkout: `/tmp/mcs-review2.cWpPdm` at
`5ef195d1d40316ef1048612d3de2c93c20e9afe2`. Product files are unchanged from
the implementation candidate; `git diff 700c520..5ef195d -- .` excluding
`.factory` and `graphify-out` is empty.

| Command | Result |
|---|---|
| `npm ci` | PASS; 0 vulnerabilities |
| `npm test` | PASS; 2 Vitest, 14 Rust, and 1 startup test |
| `npm run check` | PASS; 0 Svelte errors/warnings and strict Clippy passed |
| `npm run build` | PASS; `dist/` produced; JS 75.01 KB / 27.17 KB gzip, CSS 18.54 KB / 4.97 KB gzip |
| `npm run test:e2e -- --reporter=line` | PASS; 16/16 desktop and phone tests |
| `npm run test:claims` | PASS; all 12 declared commands |
| `BUILD_SHA=700c520… cargo build --locked --release` | PASS |

The local normal/invalid/boundary checks returned `201` for a valid minimum
configuration, `400` for interval zero, `400` for a loopback target, `401` for
an invalid access code, and `204` for deletion. A separate stop/start test kept
one disabled probe in the same temporary data directory and reported both
migrations current with persisted key material.

## Live product verification

- `/`, `/health`, `/demo`, `/privacy`, `/terms`, `robots.txt`, `sitemap.xml`,
  favicon, and Apple touch icon returned 200. The deliberately unknown route
  correctly returned a designed HTML 404 with a title, one `h1`, `main`, and a
  home action.
- Desktop and phone sample flows showed two realistic probes, attributed 429
  evidence, an open alert, and a recovered alert. Reset removed temporary demo
  edits. **Start for real** returned to the separate access-code screen. No
  `/api/*` or cross-origin request occurred during either demo flow.
- Keyboard order began with the skip link and reached the sample action; Enter
  opened the demo. Each focused item had a 3 px blue outline. At 200% root text
  size the phone page retained its heading, banner, actions, and no horizontal
  overflow.
- Reduced motion matched the media query, disabled smooth scrolling, and reduced
  the specimen transition to `0.00001s`. An offline reload retained the demo,
  title, and banner with no console error.
- Live axe scans returned zero violations on desktop and phone. The factory URL
  verifier found one `h1`, one `main`, `lang=en`, complete image alternatives,
  labelled buttons, and no console error on `/` and `/demo`.
- Every rendered internal and external link returned 200. Route titles are
  distinct. Security headers include CSP, frame denial, no-sniff, and
  `Referrer-Policy: no-referrer`. Hashed JS/CSS responses are immutable.
- An explicit invalid Atlas restore made one request to the documented Sociobot
  verification origin and showed “License no longer active.” No request was
  made before that user action.
- A 220-request live invalid-access burst returned 46 × 401 and 174 × 429.
  Every 429 had `Retry-After: 1`; a request after 1.2 seconds returned 401.
  Sixty `/health` requests returned 200.
- Lighthouse mobile scores were 100 performance, 100 accessibility, 100 best
  practices, and 100 SEO. LCP was 1.73 s, total blocking time 35 ms, CLS 0, and
  total transfer 162,584 bytes.

The locally rebuilt HTML, JS, and CSS using served build SHA `784a4d3…` are
byte-identical to the live assets.

The missed-leverage review found no absent AI action: this product observes
model endpoints, and adding generated analysis would add cost without improving
the brief's deterministic capacity and output-shape checks.

## Deployment and rollback evidence

Only product-scoped Container App, revision, replica, and image metadata were
read. No configuration, traffic, revision, data, secret, or other service was
changed.

| Check | Result |
|---|---|
| ARM provisioning | `Succeeded`; running status `Running` |
| Revision mode | `Single` |
| Active revisions | Exactly one: generated `sf-model-capacity-sentinel--0000009` |
| Active health | `Healthy`, `Provisioned`, `RunningAtMaxScale`; one running replica |
| Traffic | `latestRevision: true`, weight `100`; no named target |
| Scale and storage | min 1, max 1; existing Azure Files volume mounted at `/data` |
| Served build | `/health` returns `784a4d372874016fcdce97271a18c7d79afcbd0a` |
| Current immutable image | `sha256:a0e31519ca5b577a152375cdfa1fe91e6e3cc1c6cf200f44f2f04f645946b4c7` |
| Compatible rollback image | `sha256:6e28640d4bb7d4118f2542cd67c4ae06d883ac26e940a53c0d163f796d16d1ed` |

The current digest exists with tag `784a4d372874`; the rollback digest exists
with implementation tag `700c520391446b89b7ca82570ccb5e9cb7cbf716` and is the
stopped healthy generated revision `0000008`.

The five manually named revisions `sqlite-retry`, `sqlite-safe`, `schema-safe`,
`schema-diag`, and `recovery-safe` are all inactive, stopped, have zero replicas,
and carry zero traffic. No named revision exists after `recovery-safe`; the only
later revisions are generated `0000008` and `0000009`.

## Earlier finding disposition

| Earlier finding | Current disposition |
|---|---|
| Review 1 F1, missing demo | Fixed; clean and live one-click demo, reset, label, and isolation pass. |
| Review 1 F2, no claim manifest | Partly fixed; 12 declared commands pass, but F1 above identifies 12 remaining untested claim families. |
| Review 1 F3, inert edit | Behavior fixed; pointer and keyboard editing pass. History preservation remains incompletely asserted under F1. |
| Review 1 F4 / Verification 4, rate limit | Fixed; live 429 and positive `Retry-After` pass. |
| Review 1 F5, broken checkout | Broken checkout offer removed and pending registration stated honestly. Invalid restore fails closed; complete Atlas claims remain under F1. |
| Review 1 F6, first-screen copy | Fixed on desktop and phone. |
| Review 1 F7, routes and metadata | Fixed; all routes, titles, metadata, links, and designed 404 pass. |
| Review 1 F8, pinned Rust base | Fixed; Dockerfile uses `rust:1-slim`; locked release build passes. |
| Review 1 F9, phone targets | Fixed; no visible phone target measured below 44 × 44 px. |
| Verification 1, public writes/plaintext/populated axe | Fixed in observed behavior and source tests. |
| Verification 1, immutable asset caching | Fixed; hashed JS/CSS return one-year immutable caching. |
| Verification 3, identity/console/startup record | Fixed; served identity, console, and structured startup record pass. |
| Verification 5 PASS | Runtime behavior reconfirmed; strict claim audit now supersedes its verdict. |
| Verification 6, failed restoration/outage | Fixed; healthy single revision and both HTTPS endpoints pass. |
| Verification 7 PASS | Infrastructure and behavior reconfirmed; its zero-untested-claims conclusion is superseded by F1. |

## Evidence

Screenshots, URL-verifier output, and Lighthouse JSON are under
`/work/.evidence/` with the `review-2-` prefix. No live project data was opened
or changed; all writes used browser-local demo storage or a temporary local
SQLite directory.
