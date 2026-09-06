# Isolation report — Capacity Sentinel repair 5

Run date: 2026-09-06 UTC. Scope was limited to the
`sf-model-capacity-sentinel` Container App, its existing revisions, and its
product image repository. No other app, database, staging slot, or secret
value was read or changed.

## Requested restoration

- Existing target selected: `sf-model-capacity-sentinel--recovery-safe`, the
  newest existing revision, using immutable image digest
  `sha256:afe74de9245868781bc8d91aa926a7495ba7111da1a71175367b25f769641d9b`.
- ARM activation of that already-existing revision succeeded. No revision was
  created and no manual revision suffix was added.
- App-level revision mode was already `Single`. Its single-mode ingress was
  already exactly `latestRevision: true, weight: 100`; Azure rejects a named
  traffic target in this mode. With `latestRevisionName` set to
  `recovery-safe`, that is the only traffic target.
- The existing product topology remained unchanged: one mounted Azure Files
  volume at `/data`, no configured probes, and `minReplicas: 1` /
  `maxReplicas: 1`. Environment and secret values were not read.

## Redacted ARM evidence after activation

| Check | Result |
|---|---|
| Container App provisioning state | `Failed` — not restored to `Succeeded` |
| Revision mode | `Single` |
| Active revisions | Exactly one: `recovery-safe` |
| Active revision state | `Activating`, `healthState: None`, one allocated replica |
| Replica state | `CrashLoopBackOff` before ready |
| Ingress | Sole single-mode target: `latestRevision: 100` |
| Storage / scale | Azure Files `/data` retained; one minimum and one maximum replica |
| HTTPS `/` | Timed out, no HTTP response (`000`) |
| HTTPS `/health` | Timed out, no HTTP response (`000`) |

The selected image maps to build SHA `90109876b120284fa7390a7342774a37c605b452`,
but it was not served because the process crash-looped during schema
initialization before opening its port. Therefore there is no served build SHA
or successful health response to report.

The previous declarative revision's rollback image digest is
`sha256:30b314916152b24efc1e1a4c92ca316461d3f8c754e23ed5f56f80a3c2c3abe1`.
It was not activated: this report does not claim it is a safe rollback, only
records the pre-existing digest without changing product code or topology.

## Guardrail added

`README.md` now requires one declarative Container App app update for normal
deployments and prohibits hand-created/copy revisions, manual suffixes, and
named traffic management. It also documents Azure's `latestRevision: 100`
representation in single-revision mode and the required non-secret
verification checks.
