# Verification report — FAIL

Verified 2026-08-28 against candidate `8bdec478d62084ccdf7fcd5bbfe46cfe6f21e8b9` on `main` and the live URL <https://model-capacity-sentinel.sociobot.in>.

## Decision

**FAIL.** The candidate has passing repository checks and its deployed frontend is byte-identical to the local production build, but it does not meet the security/privacy and accessibility acceptance contract.

## What passed

- Clean checkout was exactly `8bdec478d62084ccdf7fcd5bbfe46cfe6f21e8b9`; `npm ci` completed with 0 audit vulnerabilities.
- `npm test`: 2 Vitest tests and 4 Rust tests passed.
- `npm run check`: Svelte check reported 0 errors/0 warnings; `cargo clippy -- -D warnings` passed.
- `npm run build`: passed. Initial JS is 66,280 bytes raw / 24,990 gzip; CSS is 16,991 bytes raw / 4,660 gzip; hero WebP is 129,198 bytes. All meet the specified static budgets.
- `npm run test:e2e`: 2/2 repository Playwright tests passed (desktop and 390×844 mobile).
- Independent local end-to-end API/UI checks passed for a successful OpenAI-compatible JSON canary, 429 capacity classification and alert after two runs, invalid JSON, missing JSON invariant, timeout, daily cost cap, input-range rejection, UI error/retry recovery, rate limiting (53 creates accepted then 8 HTTP 429 when the 60-write window was exhausted), and 100 concurrent `/health` requests (100 HTTP 200).
- API-key ciphertext did not expose the supplied test key in SQLite strings; the generated `master.key` was mode `0600`.
- Local service worker controlled the page and an offline reload of `/privacy` succeeded.
- Live desktop and 390px mobile empty-state pages had one `h1`, no console/page errors, no third-party load origins, and no serious/critical axe findings. `/privacy`, `/terms`, and API endpoints returned expected HTTP statuses. CSP, `X-Content-Type-Options`, `X-Frame-Options`, and `Referrer-Policy` headers were present.
- The deployed `index-DwpLRRWf.js`, `index-BXDZOstr.css`, and hero WebP SHA-256 hashes exactly matched the candidate's local `dist` artifacts. This confirms the live frontend content matches this candidate.

## Blocking defects

### High — public unauthenticated write API permits cross-tenant data destruction and SSRF

`/api/summary`, `/api/probes`, `/api/probes/{id}`, and `/api/probes/{id}/run` have no authentication or ownership boundary. The live product is publicly reachable. Any network client can list another user's probe configuration and synthetic prompt, delete probes/history, or create probes. Endpoint validation accepts arbitrary `http` and `https` addresses, including `http://127.0.0.1`; the local verification created such a probe successfully (HTTP 201) and the runner made the request. On a public deployment this is an SSRF primitive against the host/network, in addition to a cross-tenant integrity failure.

Required remediation: do not expose a shared instance without authentication/tenant isolation; bind all API routes to an authenticated project, and reject loopback, link-local, private, and otherwise disallowed resolved addresses (including redirect revalidation) or use an explicit endpoint allowlist.

### High — synthetic prompts are retained in plaintext, contrary to the researched privacy contract

The brief specifies comparable evidence “without retaining prompts.” `migrations/0001_init.sql` defines `probes.prompt TEXT NOT NULL`; `/api/summary` returns it; local SQLite/WAL inspection showed the exact test values `Synthetic QA probe only` and `Synthetic QA capacity only` in plaintext. The app's narrower “without retaining production prompts” copy does not meet the supplied acceptance contract.

Required remediation: do not persist the prompt text (or store it only in a user-controlled encrypted secret with an explicit retention model); retain only the evidence required by the brief.

### High — serious axe failures on every populated probe dashboard

Independent axe 4.10.2 scans on both desktop and 390px populated local dashboards reported `aria-prohibited-attr` at serious impact. Each trend container is emitted as e.g. `<div class="ticks" aria-label="2 consecutive failures">`; a plain `div` cannot take `aria-label` without a valid role. The repository test only scans the empty state, so it misses this required state.

Required remediation: give the element an appropriate semantic/ARIA role with an accessible name, or replace it with valid text/visually-hidden text. Add a populated-dashboard axe regression test.

## Other defects / gaps

### Medium — live backend build identity cannot identify the candidate

The live `/health` response was `{"build":"container","status":"ok"}`, not the tested SHA. Although static artifact hashes match, backend identity cannot be confirmed from the health endpoint as required. Deployment must pass `--build-arg BUILD_SHA=8bdec478d62084ccdf7fcd5bbfe46cfe6f21e8b9` (or equivalent immutable release ID) and expose it through `/health`.

### Medium — static response caching policy misses the stated immutable-cache requirement

Live HTML, hashed JS/CSS, service worker, and hero image responses had no `Cache-Control` header. This misses the factory requirement for long-lived immutable caching of hashed assets and makes browser cache behavior deployment-dependent.

### Test limitation — container image build not run in this environment

`docker` is not installed in the verification container, so `docker build` could not be executed. Native build/tests and the deployed artifacts were verified instead. This is not the basis of the FAIL decision.

## Reproduction commands

```bash
npm ci
npm test
npm run check
npm run build
npm run test:e2e

curl -i https://model-capacity-sentinel.sociobot.in/health
curl -sS https://model-capacity-sentinel.sociobot.in/assets/index-DwpLRRWf.js | sha256sum
sha256sum dist/assets/index-DwpLRRWf.js
```

For the accessibility failure, seed any valid local probe and run an axe scan on the populated dashboard; it reports `aria-prohibited-attr` on `.ticks[aria-label]`.
