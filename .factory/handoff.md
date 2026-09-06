# Capacity Sentinel repair 4 handoff

Implementation candidate: `90109876b120284fa7390a7342774a37c605b452`.

## What changed

- Added `/demo`, an isolated sample workspace, a persistent demo label, reset,
  and a path back to real data. Demo actions never call the API or use the
  real storage namespace.
- Added the twelve-entry claims manifest, the documented claim runner, and
  outcome-based browser/backend tests for every public claim.
- Made probe editing work from mouse and keyboard, including a populated
  sample probe and reset path.
- Enforced rate limits before access-code validation. Invalid public API
  traffic now receives `429` and a positive `Retry-After` after its allowance.
- Removed the unregistered Atlas checkout link. Atlas remains a $39 one-time
  paid 365-day comparison; billing registration is an external dependency and
  the application now says so rather than sending visitors to a 404.
- Rewrote the first screen, catalog description, metadata, route titles,
  designed 404, mobile targets, accessibility states, legal routes, and
  product-specific demo/claims documentation.
- Updated the container build to `rust:1-slim`, used build arguments without
  `.git`, and retained non-root runtime behavior.
- Made durable SQLite startup safe for a one-replica Azure Files handoff:
  startup no longer changes journal mode, validates an already-current schema
  read-only, and preserves a locked empty primary database while initializing
  a separate durable recovery database only in that condition.

## Verification

```text
npm test                         PASS
npm run check                    PASS (Svelte: 0 diagnostics; Clippy -D warnings)
npm run build                    PASS (75.01 kB JS / 27.17 kB gzip; 18.54 kB CSS)
npm run test:claims              PASS
npm run test:e2e -- --reporter=line  PASS (16/16)
cargo test                       PASS (12 unit/integration + startup persistence)
cargo build --locked --release   PASS
npm audit --omit=dev             PASS (0 vulnerabilities)
```

The SQLite regression test `current_schema_validation_does_not_require_a_database_write`
sets a migrated database read-only and proves an ordinary restart only validates it.
The local URL verifier found title, language, one h1, main, image alternatives,
button labels, and console checks clean on `/` and `/demo`; the browser suite
also runs axe checks, keyboard paths, 390 px layout, 404, reset, and rate-limit
recovery.

## Deployment and live check

The product image for the implementation candidate built successfully in the
product registry as
`sha256:afe74de9245868781bc8d91aa926a7495ba7111da1a71175367b25f769641d9b`.
It could not be promoted: this product's mounted Azure File share rejects the
first SQLite `CREATE TABLE` on both the existing empty `sentinel.db` and a
fresh `sentinel-recovery.db` with `database is locked` after 30 seconds. This
was reproduced with all failed product revisions stopped; neither file was
deleted or overwritten. The prior healthy revision was restored and its cold
HTTPS health check returns build `8793fb3538ebd965d259ff8248468df3a64a504c`.

The implementation candidate therefore remains committed and buildable but is
not the live image. The mounted SQLite storage issue needs a compatible
factory-provided `/data` filesystem before it can be promoted without violating
the durable-state contract. No external or non-product resource was inspected
or changed.

## Known dependency

Atlas billing registration is still controlled by the separate Sociobot billing
operator. The paid deliverable and terms remain present, but checkout is not
advertised until that registration returns a real hosted checkout. The required
public offer metadata is at `/work/.evidence/billing-offer.json` in the worker
evidence area; it contains no credentials.
