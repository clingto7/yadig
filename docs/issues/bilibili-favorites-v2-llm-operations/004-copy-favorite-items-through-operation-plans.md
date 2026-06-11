# Issue 4: Copy favorite items through operation plans

## Parent

PRD: `docs/prd/PRD-bilibili-favorites-v2-llm-operations.md`

## What to build

Add copy favorite as a first-class operation plan and execution path. The user should be able to select favorite items, choose a target folder, generate a copy draft, preview source/target/item effects, explicitly confirm, and execute against Bilibili while preserving source membership.

Copy should reuse the safety model already established for move/delete: complete write session required, valid source/target/resource identity required, small batches, pauses between batches, sanitized errors, stop on account/security failures, and local membership update only after remote success.

## Acceptance criteria

- [ ] Operation plan kind supports Bilibili favorite copy without breaking existing move/delete/audio plans.
- [ ] Copy plan creation validates source folder, target folder, resource id, and resource type.
- [ ] Copying to the same source folder is skipped before execution.
- [ ] If local state already shows target membership, the item is skipped as an existing target membership.
- [ ] Workstation can create a copy draft from selected favorite items or from selected classification results.
- [ ] Draft preview shows action, source folder, target folder, item count, affected titles, skipped items, and blocked reasons.
- [ ] Execution requires explicit confirmation after preview.
- [ ] The backend calls the Bilibili favorite resource copy endpoint with source folder, target folder, current account id, resource id/type pairs, and CSRF token.
- [ ] Execution chunks copy requests by source/target folder and pauses between successful batches.
- [ ] Auth, CSRF, captcha, rate-limit, risk-control, and malformed-plan failures stop remaining execution.
- [ ] Successful copy adds local target membership and preserves source membership.
- [ ] Copy results appear in favorite operation history with per-item status and sanitized errors.
- [ ] Rust tests cover copy plan validation, form construction, batching, same-folder skip, existing-target skip, stop-on-blocked behavior, and local membership update semantics.
- [ ] TypeScript compilation, Rust tests, and production build pass.
- [ ] Manual smoke test copies one or two videos between disposable folders and confirms both source and target membership on Bilibili Web.

## Blocked by

None - can start immediately.
