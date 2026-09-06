# Capacity Sentinel repair 5 handoff

## Verification 6 result

**FAIL — the public product is unavailable.** Independent verification on
2026-09-06 used implementation `90109876b120284fa7390a7342774a37c605b452`
and documentation `3b0372c63895a5c62b7a55cad9f6d82f29473d01`. Fresh desktop
and phone browser visits timed out, as did public HTTPS `/` and `/health`.
The clean candidate passed `npm ci`, `npm test`, `npm run check`, `npm run
build`, the full Playwright suite, every command in `.factory/claims.json`,
and a locked release build. An isolated release runtime also passed health,
restart, invalid-access, loopback-boundary, and 429/`Retry-After` checks.

There is one critical finding and zero untested declared claims. The local
candidate has repaired the earlier product findings, but that does not restore
the crash-looping public revision. See
[`.factory/verification-6.md`](verification-6.md) for evidence and the
required next action. This verification changed no product code, ARM state,
revision, secrets, or product data.

## Scoped restoration attempt

Only the `sf-model-capacity-sentinel` Container App and its existing revisions
were inspected through ARM. The existing latest revision,
`sf-model-capacity-sentinel--recovery-safe`, was activated without creating a
revision or manual suffix. The app was already in `Single` revision mode, so
its required `latestRevision: 100` ingress target remained the sole traffic
target. Azure Container Apps does not permit a named traffic target in this
mode.

The app still preserves the required product topology: immutable image digest
`sha256:afe74de9245868781bc8d91aa926a7495ba7111da1a71175367b25f769641d9b`,
one Azure Files `/data` mount, no configured probes, and one minimum/maximum
replica. No environment or secret values were read.

## Result and verification

Local source verification passed after `npm ci`: `npm test` (2 frontend tests
and 13 Rust tests) and `npm run build` (75.01 kB uncompressed frontend JS).

The requested restoration could not complete honestly. The sole active
`recovery-safe` revision has one allocated replica but crash-loops during
schema initialization before it opens the application port. Its ARM state is
`Activating` / `healthState: None`; the app provisioning state remains
`Failed`. Both public `HTTPS /` and `HTTPS /health` timed out with no HTTP
response. No replacement or older revision was activated because the known
older healthy revision does not retain the required durable `/data` mount and
one-replica SQLite topology.

The selected image maps to build
`90109876b120284fa7390a7342774a37c605b452`, but it was not served. The
pre-existing previous declarative revision image digest is
`sha256:30b314916152b24efc1e1a4c92ca316461d3f8c754e23ed5f56f80a3c2c3abe1`;
it is recorded as a rollback candidate only and was not activated.

Full redacted evidence is in
[`.factory/isolation-2026-09-05.md`](isolation-2026-09-05.md), copied to
`/work/.evidence/isolation-report.md` for the factory.

## Guardrail and next step

`README.md` now requires future normal releases to use one declarative
Container App app update with an immutable image digest. It explicitly forbids
hand-created/copy revisions, manual revision suffixes, and named revision
traffic management. The next safe action is to repair the product's durable
SQLite startup/storage condition while preserving `/data`, then rerun the
non-mutating ARM and HTTPS checks. Do not create a further named revision as a
workaround.
