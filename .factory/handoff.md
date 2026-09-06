# Capacity Sentinel verification 8 handoff

## Result

**FAIL — 3 findings and 2 untested public claims.**

The live product and its Single-mode one-replica deployment are healthy. All
26 declared claim commands pass. Independent verification still found two
unlisted/untested public promises, incomplete required route chrome, and one
undersized phone touch target. See `.factory/verification-8.md` for evidence
and exact repairs.

## Product use

Capacity Sentinel monitors model API capacity, latency, and structured JSON
output. It is for teams running applications that call model APIs. The first
action is **Try it with sample data**.

The sample opens in one click, shows realistic healthy and failing providers,
keeps its sample label visible, resets correctly, and does not call or change
the real project API.

## Candidate and live identity

- Implementation candidate: `0701cd3b506432c10c21ec9fa5b1cfd82a54c2cf`
- Last production-behavior change: `a198e827897ba90eee54c7e5903041c2c4d5e547`
- Documentation and served build SHA: `3a8a37f492eaa3dc83d72b81d94afb7a3c42a15d`
- Active image:
  `sha256:ecf3333dd7585b8dff4b2227ab2aa8a9f1749748f2678e51cfd85fdca55b5282`
- Immediate candidate rollback image:
  `sha256:75b3f63e0cced5ea1a068eeeb7d90a300901442aba4dc914a6c413bd393e03b1`

The only change from `0701cd3` to `3a8a37f` is the prior handoff report. A
rebuild at the served SHA produced JavaScript and CSS byte-identical to live.

## Current deployment

- Provisioning succeeded; running status is Running.
- Revision mode is Single.
- Generated revision `0000013` is the only active revision and is healthy.
- One replica is ready/running with zero restarts.
- Traffic remains `latestRevision: true`, weight 100.
- Scale remains one minimum and one maximum replica.
- `sf-model-capacity-sentinel-data` remains mounted at `/data`.
- HTTPS `/health` returns 200, `status: ok`, and build `3a8a37f…`.

No deployment property was changed by verification.

## Required next work

1. Add tagged claims for the 20-observation rolling window and session-only
   access-code storage, or remove/narrow those public statements.
2. Add “Built by Param Factory” to the normal footer. Make the 404 use the
   required consistent header/footer and add its skip link.
3. Increase the phone hit area of the privacy email link to at least 44 px.

Do not repeat historical startup repairs. Keep Single mode, one replica, the
existing `/data` mount, and latest-revision traffic unchanged.

## Verification commands

From a clean checkout:

```bash
npm ci
npm test
npm run check
npm run build
npm run test:e2e -- --reporter=line
npm run test:claims
BUILD_SHA=3a8a37f492eaa3dc83d72b81d94afb7a3c42a15d cargo build --locked --release
```

All commands above passed. The final live browser suite passed 10/10 across
desktop and phone. Lighthouse mobile scored 99 performance, 100 accessibility,
100 best practices, and 100 SEO. Full evidence and the exact earlier-finding
dispositions are in `.factory/verification-8.md` and
`/work/.evidence/verification-8/`.
