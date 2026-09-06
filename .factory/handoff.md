# Capacity Sentinel review 2 handoff

## Result

**FAIL — 1 finding and 12 untested public claim families.**

The implementation and live deployment are healthy. The blocker is the strict
claim contract: all 12 existing manifest commands pass, but the public UI,
legal pages, and README retain 12 additional or incompletely asserted claim
families. See `.factory/review-2.md` for the exact inventory and evidence.

No product code, deployment setting, revision, traffic rule, secret, or live
product data was changed during this review.

## Identities

- Implementation candidate: `700c520391446b89b7ca82570ccb5e9cb7cbf716`
- Repository documentation head reviewed: `5ef195d1d40316ef1048612d3de2c93c20e9afe2`
- Served build SHA: `784a4d372874016fcdce97271a18c7d79afcbd0a`
- Current image digest:
  `sha256:a0e31519ca5b577a152375cdfa1fe91e6e3cc1c6cf200f44f2f04f645946b4c7`
- Compatible rollback digest:
  `sha256:6e28640d4bb7d4118f2542cd67c4ae06d883ac26e940a53c0d163f796d16d1ed`

Only `.factory` reports differ after the implementation candidate. A local
build made with the served SHA produced HTML, JS, and CSS byte-identical to the
live deployment.

## Verification summary

- Fresh clone at `5ef195d`: `npm ci`, `npm test`, `npm run check`,
  `npm run build`, 16 Playwright tests, all 12 declared claim commands, and the
  locked release build passed.
- Fresh desktop and 390 px phone browsers passed the first-screen, sample,
  reset, start-for-real, keyboard, focus, reduced-motion, 200% text, axe,
  offline reload, link, legal route, metadata, and deliberate 404 checks.
- Demo activity stayed same-origin, made no project API request, and changed no
  real data.
- Local temporary-runner checks passed valid, invalid, boundary, delete, health,
  and stop/start persistence paths.
- Live invalid-access load returned 429 with `Retry-After: 1` and recovered to
  401 after the allowance replenished.
- Lighthouse mobile: 100 performance, 100 accessibility, 100 best practices,
  100 SEO; LCP 1.73 s, TBT 35 ms, CLS 0.

## Deployment state

- ARM `Succeeded`; revision mode `Single`.
- Exactly one active generated revision:
  `sf-model-capacity-sentinel--0000009`, healthy, one running replica.
- Traffic is `latestRevision: 100` with no named target.
- Scale remains one replica and the existing Azure Files volume remains mounted
  at `/data`.
- The five named recovery revisions remain inactive at zero traffic. No new
  named or hand-managed revision was created.
- `/` and `/health` return 200.

## Next step

Add or narrow claim entries and complete tagged sandbox tests for the 12 claim
families in `.factory/review-2.md`. Re-run the same clean-checkout and live
review without changing the currently healthy revision topology.
