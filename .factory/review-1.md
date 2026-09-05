# Review 1: Monitor model API capacity and output shape

Review date: 2026-09-05 UTC  
Live URL: <https://model-capacity-sentinel.sociobot.in>  
Implementation reviewed: `301106fc3e2fbafc493fc2d34c7b5e5d45efe63b`  
Live image build label: `8793fb3538ebd965d259ff8248468df3a64a504c`  
Documentation reviewed: `934e23bcc9ac36cab24fe9ace2b6649ba7d8e83a`

## Verdict

**FAIL — 9 findings and 26 untested public claims.**

The implementation commit is `301106f`. The next two commits changed only
`.factory` reports, so the live build label at `8793fb3` and the current
documentation head at `934e23b` contain the same product implementation.
The live HTML, JavaScript, and CSS are byte-identical to the clean build.

The core authenticated backend works in local tests, but this release does not
provide the required sample sandbox. A visitor cannot see realistic output or
try the job without a project access code. The edit action is also inert, the
paid checkout is broken, and unauthenticated live API requests do not reach the
required rate limit.

## Job, audience, and first action before scrolling

- Job: send synthetic requests to model APIs and detect capacity, latency, or
  JSON-shape failures before production users report them.
- Audience: teams that operate applications using one or more model APIs.
- First useful action required by the contract: **Try it with sample data**.
- Actual first action on desktop and phone: **Plant a canary**. It opens an
  empty configuration form. The page does not offer sample data and does not
  name the team audience in the first screen.

## Findings

### F1 — High — The required one-click sample sandbox is absent

There is no “Try it with sample data” action, `.factory/demo.md`, sample label,
reset action, start-for-real action, demo storage namespace, or ephemeral demo
tenant. `GET /demo` returns an empty 404. `/?demo=1` loads the normal locked
landing page with no demo banner.

This prevents a visitor and reviewer from seeing a realistic populated result
without a real project access code. The required checks for reset and isolation
from real data therefore cannot run. No live product data was changed during
this review; populated checks used a temporary local SQLite directory.

Required repair: add a one-click `/demo` workspace with realistic providers,
healthy and failing observations, open and recovered alerts, a persistent
“Demo — sample data, nothing is saved” label, reset, and start-for-real.

### F2 — High — Public claims have no required claim manifest or tagged tests

`.factory/claims.json` does not exist. No test contains an `@claim:` tag. The
landing page, legal pages, and README make 26 distinct user-facing claim
families, but none has the required clean-sandbox command. Existing unit and
integration tests provide useful partial evidence; they do not satisfy the
claim contract because they are not declared, uniquely tagged, or run through
the sample entry point.

The 26 untested claim families are:

1. scheduled synthetic requests;
2. availability measurement;
3. elapsed-latency and rolling p95 measurement;
4. named JSON-path validation;
5. 429 capacity classification;
6. timeout, network, upstream, invalid-JSON, and invariant classification;
7. attributed alerts after two failures;
8. automatic alert recovery;
9. AES-256-GCM API-key encryption;
10. encrypted canaries that are never returned or redisplayed;
11. production prompts never passing through the service;
12. daily token caps;
13. response-token limits;
14. blocking private, loopback, link-local, and redirected endpoints;
15. local SQLite storage and local operational data;
16. unrestricted CSV export;
17. keyboard-ready operation;
18. mobile-ready operation;
19. offline, error, and empty states;
20. access-code protection for every API endpoint;
21. per-client API limits with `429` and `Retry-After`;
22. no analytics and no third-party call except optional licensing;
23. deletion of a probe and its local history;
24. a working `$39` one-time Atlas purchase and restore flow;
25. the 365-day Atlas comparison view; and
26. container startup, non-root execution, `/data` persistence, and immutable
   health identity.

Required repair: add one manifest entry and one observable `@claim:<id>` demo
test for every retained claim. Remove claims that cannot be proved.

### F3 — High — The edit action does nothing

On a populated local dashboard, clicking the accessible button named “Edit
Review capacity specimen” left the edit dialog closed and focus on the button.
Source inspection confirms the edit button has no click handler. Operators
cannot change an endpoint, model, key, schedule, objectives, or token limits
without deleting the probe and losing its history.

Required repair: connect the edit button to the existing edit dialog, cover
keyboard and pointer activation, preserve encrypted fields when left blank,
and add an end-to-end regression test.

### F4 — High — Live unauthenticated API traffic is not rate limited

An 80-request concurrent burst to live `/api/summary` with an invalid bearer
token returned **80 × 401**, **0 × 429**, and no `Retry-After`. This leaves the
public access-code check outside the effective live allowance and permits
unbounded guessing at that boundary.

Authenticated local limits work: a 41-request read burst returned 40 × 200 and
1 × 429; a 21-request invalid write burst returned 20 × 422 and 1 × 429. Both
local 429 responses had `Retry-After: 1`. That does not satisfy the contract
that every server-side API request is limited at the live boundary.

Required repair: apply the per-client limiter before access-code validation,
add an unauthenticated burst regression, deploy, then prove live 429 responses
with a positive `Retry-After`.

### F5 — Medium — The paid checkout link is broken

The page offers “Unlock Atlas” and “Buy Atlas — $39 once”. A fresh GET to the
linked URL returned HTTP 404 with
`{"error":"enabled factory product","status":404}`. The source link returned
200, and the internal privacy and terms links returned 200.

Required repair: register or enable this product in the Sociobot billing
service, then test checkout return, token removal from the URL, verification,
restore, invalid/revoked handling, and cached offline first paint.

### F6 — Medium — The first screen does not state the job, audience, or sample action

The heading “Notice the change before users do” does not name the model API
monitoring job. The next sentence lists behavior but does not name the team
audience. “Plant a canary” and section headings such as “Privacy, by habitat”
use the field-guide metaphor instead of the required plain task language. The
primary action also opens a form before the visitor has supplied the access
code needed to save it. `.factory/copy-audit.md` is missing.

Required repair: use a job-naming heading, name model API operators in the
audience sentence, make the sample the primary action, use plain section and
control names, and add the required copy audit.

### F7 — Medium — Required routes and metadata are incomplete

- `/privacy` and `/terms` keep the landing title instead of route-specific
  titles.
- Unknown paths return the expected HTTP 404 status, but the response is empty:
  no title, `h1`, `main`, product styling, or way home.
- `robots.txt`, `sitemap.xml`, and the Apple touch icon return 404.
- The document has no canonical link, Open Graph metadata, Twitter card, or
  1200 × 630 social image.
- Footer output has no version/build identifier.
- Route changes use full document navigation and do not focus or announce the
  destination heading.

Required repair: implement the required route skeleton, metadata assets,
designed 404 body, route titles, navigation focus announcement, and build ID.

### F8 — Medium — The Docker build pins a Rust minor release

The Dockerfile uses `FROM rust:1.90-bookworm`. The backend contract explicitly
requires a floating stable `rust:1-slim` or `rust:1-alpine` image and forbids a
minor pin because current locked dependencies can require newer Rust releases.
Docker is not installed in this worker, so an image build was not run; the
native locked release build passed.

Required repair: use the required stable Rust base tag and run the multi-stage
image build from a source archive without `.git`.

### F9 — Low — Several phone touch targets are below 44 px

At 390 px width, “Read the field method” measured 40.8 px high. Footer links
measured 18.6 px high and 35.3–43.5 px wide. These miss the 44 × 44 px touch
target baseline. Adjacent footer links have spacing, but their targets remain
too small.

Required repair: enlarge the interactive hit areas to at least 44 × 44 CSS px
without changing the visible text size.

## Live browser evidence

Fresh Chromium contexts were used at 1440 × 900 and 390 × 844.

| Check | Result |
|---|---|
| Landing response, title, language, landmarks | 200; title present; `lang=en`; one `h1`; one `main` |
| First viewport | No sample action; no audience statement; form-first action |
| Console and page errors | None |
| External requests on first load | None |
| Horizontal overflow at normal size | None on desktop or phone |
| Keyboard start | Skip link first; visible 3 px blue focus outline |
| Empty-state axe | Zero serious or critical findings on desktop and phone |
| Populated-state axe | Zero serious or critical findings locally on desktop and phone |
| Reduced motion | Transition duration `0.00001s`; smooth scrolling disabled |
| Service worker update | Completed; page controlled |
| Offline reload | `/privacy` reloaded with status 200 after being cached; no errors |
| Privacy controls | Privacy page names stored data, deletion, external license call, and contact email |
| Unknown route | Expected 404 status, but empty response; F7 |

The factory URL verifier passed with load time 657 ms, no console errors, title
and language present, one `h1`, one `main`, no missing image alternative, and
no unnamed button. The separate axe CLI could not start because its downloaded
ChromeDriver was version 152 while the supplied Chromium was version 145. The
required Playwright axe integration ran successfully instead.

## Local backend and populated-state evidence

All local data lived under `/tmp/capacity-review.qPoCFc`; no live project data
was read or changed.

- Wrong access code: 401. Correct code recovered through Enter and showed two
  populated probes.
- Loopback endpoint: 400. Interval zero: 400. Minimum allowed schedule and
  budgets: accepted; execution stopped at the configured cost cap.
- Two safe requests to `https://httpbin.org/status/429` produced `capacity`
  observations and an attributed open alert after the second run.
- CSV returned the expected header and one row per observation.
- Restart with the same directory retained 2 probes, 4 observations, and 2
  alerts. Startup logged `master_key_source=persisted` without secret values.
- Raw SQLite/WAL string inspection found neither the test key nor canary text;
  the generated master key mode was `0600`.
- The realistic populated dashboard showed summary metrics, two attributed
  alerts, two provider/model rows, details, objectives, and history-derived
  values on desktop and phone. It had no overflow, console errors, or serious
  axe findings. The inert edit action is F3.
- A fully healthy OpenAI-compatible live response was not exercised because
  the product has no demo and no provider credential was supplied. This is
  included in the untested claim count in F2.

## Clean-checkout commands

The clean clone was at documentation SHA `934e23b`; its product files are
identical to implementation SHA `301106f`.

| Command | Result |
|---|---|
| `npm ci` | PASS; 134 packages; 0 vulnerabilities |
| `npm test` | PASS; 3 Vitest, 10 Rust unit/integration, 1 startup test |
| `npm run check` | PASS; Svelte 0 errors/warnings; Clippy clean |
| `npm run build` | PASS; `dist/` produced |
| `npm run test:e2e -- --reporter=line` | PASS; 6/6 desktop and phone tests |
| `BUILD_SHA=8793fb3… cargo build --locked --release` | PASS |
| `npm audit --omit=dev` | PASS; 0 vulnerabilities |
| documented `oha -z 10s -q 100 /health` | PASS; 1,000 HTTP 200 and 2 deadline-aborted; 100.18 req/s; 0.705 ms mean |
| Docker image build | NOT RUN; Docker is unavailable; Dockerfile violation is F8 |

Production output is 67,533 bytes JavaScript (25,075 gzip), 16,991 bytes CSS
(4,644 gzip), and 129,198 bytes for the hero WebP. Fresh mobile Lighthouse was
95 performance, 100 accessibility, 100 best practices, and 100 SEO; LCP was
1.80 s, CLS 0, and total blocking time 239 ms.

## Live implementation match

`GET /health` returned the live build label `8793fb3538ebd965d259ff8248468df3a64a504c`.
That commit differs from implementation `301106f` only in `.factory/handoff.md`.
Current documentation `934e23b` adds only review records after that. Live and
clean-build hashes match:

| Artifact | SHA-256 |
|---|---|
| `index-BrAiOFsX.js` | `824a6477eb34c5ed3b1d90471d1bbc9d9a325b6c7fa5e72bfa8305dae8869fe5` |
| `index-BXDZOstr.css` | `3e8cba760e347bf4040dc65da060e95bd58f7220c15602b7fa3a6cd21a9a8162` |
| `index.html` | `34f5cf021881ae85942d1612de6150df1d546cbe90a8a06005efd1ca666e1a5b` |

## Earlier findings

| Earlier finding | Current disposition |
|---|---|
| Unauthenticated shared writes and SSRF | Fixed for the single-project model: API requires an access code; loopback/private targets are rejected. |
| Plaintext synthetic canaries | Fixed: new and migrated canaries are encrypted; raw storage check passed. |
| Populated dashboard ARIA error | Fixed: trend marks use `role=img`; populated axe scans pass. |
| Unknown/live build identity | Fixed: live health and asset hashes identify the deployed implementation. |
| Missing immutable cache headers | Fixed for hashed JS/CSS; HTML and service worker use no-cache. |
| First-visit 401 console error | Fixed: no summary request occurs before an access code exists. |
| Missing generated/supplied startup record | Fixed and covered by the passing startup test. |
| Missing read/per-client limits and `Retry-After` | Authenticated local behavior is fixed. Live unauthenticated traffic still bypasses the allowance; see F4. |

## Evidence files

Browser data, screenshots, Lighthouse JSON, backend JSON, and verifier output
are in `/work/.evidence/`. The required report copy is
`/work/.evidence/qa-report.md`; the machine verdict is
`/work/.evidence/qa-result.json`.
