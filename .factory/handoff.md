# Capacity Sentinel repair 6 handoff

## Result

**PASS.** The public product is restored. The implementation is
`700c520391446b89b7ca82570ccb5e9cb7cbf716`, and the deployed immutable image
is `sha256:6e28640d4bb7d4118f2542cd67c4ae06d883ac26e940a53c0d163f796d16d1ed`.
The final documentation commit is report-only relative to that implementation.

Fresh public checks returned HTTP 200 for `/` and `/health`. Health served:

```json
{"build":"700c520391446b89b7ca82570ccb5e9cb7cbf716","status":"ok"}
```

## Cause and repair

The active `recovery-safe` replica failed before binding port 8080. Its log
showed that `_sqlx_migrations` was absent and the first schema write waited 30
seconds before SQLite returned code 5, `database is locked`. The earlier
fallback database was also schema-empty, so no project rows were bypassed.

Azure Files retained or could not service SQLite's default POSIX byte-range
lock path. The smallest product change opens the same durable database through
SQLite's built-in `unix-dotfile` VFS and limits the pool to one connection,
matching the required one-replica topology. No database, key, token, volume,
secret, or environment value was removed or copied.

The regression test first holds a conventional SQLite exclusive lock from a
separate process and proves that the normal schema write fails with `database
is locked`. It then opens the same file through the production connection,
applies both migrations, verifies their successful records, closes the pool,
and verifies that its filesystem lock was released.

## Deployment state

The image was built by ACR from the implementation commit with
`BUILD_SHA=700c520391446b89b7ca82570ccb5e9cb7cbf716`. Deployment used one
declarative Container App image update. No revision was created, copied,
named, activated, or traffic-managed by hand.

- ARM provisioning state: `Succeeded`
- Revision mode: `Single`
- Active revisions: exactly one, `sf-model-capacity-sentinel--0000008`
- Revision state: `Healthy`, `RunningAtMaxScale`, one ready replica
- Traffic: `latestRevision: true`, weight `100`
- Scale: minimum 1, maximum 1
- Storage: existing Azure Files volume retained at `/data`
- Ingress, environment, and secrets: retained
- Pre-update image digest: `sha256:afe74de9245868781bc8d91aa926a7495ba7111da1a71175367b25f769641d9b`
- Earlier declarative image digest: `sha256:30b314916152b24efc1e1a4c92ca316461d3f8c754e23ed5f56f80a3c2c3abe1`

Both older digests remain in the product image repository. Neither is a safe
rollback for the current Azure Files state because both use the failing
default SQLite lock path. The deployed digest above is the recovery point.

## Verification

A new clone of the pushed implementation commit passed:

```text
npm ci                                      PASS (0 vulnerabilities)
npm test                                    PASS (2 Vitest, 14 Rust, 1 startup)
npm run check                               PASS (Svelte and strict Clippy)
npm run build                               PASS (dist/ produced)
npm run test:e2e -- --reporter=line         PASS (16/16)
npm run test:claims                         PASS (all 12 manifest entries)
BUILD_SHA=<implementation> cargo build
  --locked --release                        PASS
```

The ACR build also passed. Its frontend JavaScript and CSS hashes match a local
production build made with the same SHA.

Fresh desktop and iPhone browser contexts verified the job title, audience,
one-click sample, persistent sample label, realistic open and recovered alerts,
sample editing, reset, and separation from the real project. Demo use made no
`/api/*` request and produced no console or page error. Live axe scans found no
serious or critical issue at either size. Legal routes returned 200 and the
designed unknown route returned the expected 404.

A live invalid-access burst returned 41 HTTP 401 and 4 HTTP 429 responses; all
429 responses had `Retry-After: 1`. A local stop/start check against the same
data directory retained its created probe. Live Lighthouse measured 99
performance, 100 accessibility, 100 best practices, and 100 SEO; LCP was 1.8
seconds, CLS 0, and total blocking time 120 ms.

Screenshots and Lighthouse JSON are under `/work/.evidence/`. The catalog
description was copied to `/work/.evidence/catalog-description.txt`.

## Known dependency

Atlas remains a $39 one-time add-on for the 365-day comparison view. Checkout
is honestly unavailable until the separate Sociobot billing operator registers
the product. The free runner, demo, probes, alerts, and CSV export work without
billing. Public offer metadata is in `/work/.evidence/billing-offer.json`.
