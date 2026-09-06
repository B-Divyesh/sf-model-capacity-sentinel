# Independent verification 7 — PASS

Verified 2026-09-06 UTC against implementation
`700c520391446b89b7ca82570ccb5e9cb7cbf716`, immutable image
`sha256:6e28640d4bb7d4118f2542cd67c4ae06d883ac26e940a53c0d163f796d16d1ed`,
and <https://model-capacity-sentinel.sociobot.in>.

## Decision

**PASS.** The repair fixes verification 6's only finding. The public homepage
and health endpoint return HTTP 200, the new revision is healthy, and health
serves the tested implementation SHA. No release-blocking, high, medium, or low
product defect was found. All declared claims were tested.

## Job, audience, and first action

- Job: monitor model API capacity, latency, and structured output.
- Audience: teams running applications that call one or more model APIs.
- First action: **Try it with sample data**.

These appear before scrolling in fresh desktop and phone contexts. The action
opens two realistic providers with attributed capacity evidence, an open alert,
and a recovered alert.

## Crash-loop disposition

The failed replica log showed the missing migration table followed by a
30-second `CREATE TABLE _sqlx_migrations` wait and SQLite code 5, `database is
locked`. The schema-empty fallback database proved that the failure was the
Azure Files locking path rather than a conflicting applied migration.

The candidate uses SQLite's built-in dot-file VFS with one connection. A
separate-process regression reproduces failure under the conventional lock,
then proves that the production connection applies and records both migrations
and releases its lock. The deployed startup log reports two applied and two
expected migrations, `matches: true`, then listens on port 8080.

## Clean verification

A new clone of the pushed implementation SHA passed `npm ci`, `npm test`,
`npm run check`, `npm run build`, all 16 Playwright desktop/mobile tests,
`npm run test:claims`, and the locked release build with the implementation SHA.
All 12 claim manifest entries passed their exact commands.

A separate runtime used a temporary directory, created one valid disabled
probe, stopped, reopened the same SQLite data, and returned the persisted row.
No production data or credentials were read or changed.

## Live verification

- ARM: `Succeeded`.
- Revision mode: `Single`.
- Active revisions: exactly one healthy revision, generated name `0000008`.
- Replicas: one ready; minimum 1 and maximum 1 preserved.
- Traffic: `latestRevision` at 100%.
- Storage: the existing Azure Files `/data` mount is retained.
- Homepage: HTTP 200 with no-cache response policy.
- Health: HTTP 200 and build `700c520391446b89b7ca82570ccb5e9cb7cbf716`.
- Live rate limit: 4 of 45 concurrent invalid requests returned 429, each with
  `Retry-After: 1`; the remaining requests returned the expected 401.
- Asset identity: live JavaScript and CSS SHA-256 hashes match a clean build
  made with the served implementation SHA.

Fresh desktop and iPhone contexts completed the sample edit/reset path and
returned to the separate real-project access screen. The sample label remained
visible, no project API request occurred, and no console or page error appeared.
Live axe scans found zero serious or critical violations and neither viewport
overflowed horizontally. Privacy, terms, robots, sitemap, and favicon returned
200; the designed missing route returned the expected 404.

Lighthouse: performance 99, accessibility 100, best practices 100, SEO 100;
LCP 1.8 seconds, CLS 0, total blocking time 120 ms.

## Deployment and rollback record

The release used one declarative app image update. No manual revision creation,
copy, suffix, activation, or traffic edit was used. The pre-update digest was
`sha256:afe74de9245868781bc8d91aa926a7495ba7111da1a71175367b25f769641d9b`.
The older declarative digest was
`sha256:30b314916152b24efc1e1a4c92ca316461d3f8c754e23ed5f56f80a3c2c3abe1`.
Both remain available, but neither is safe on the current volume because each
uses the failed default lock path.

Atlas checkout registration remains an external billing dependency and is
already stated on the public page. It does not affect the working free core.
