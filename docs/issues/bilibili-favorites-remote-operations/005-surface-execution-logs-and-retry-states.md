# Issue 5: Surface execution logs and retry states

## Parent

PRD: `docs/prd/PRD-bilibili-favorites-remote-operations.md`

## What to build

Expose operation plan history and execution results in Workstation so the user can audit what happened after remote account writes. The UI should show plan status, item status, successful operations, skipped items, failed items, blocked execution, and sanitized error messages. The user should be able to understand whether a failure is retryable or requires manual action.

This slice should not add new remote write behavior. It completes the feedback loop for move/delete executors.

## Acceptance criteria

- [x] Workstation shows recent Bilibili favorite operation plans with kind, status, created time, and item count.
- [x] A plan detail view shows per-item status: pending, running, success, skipped, failed, or blocked.
- [x] Result details include the source folder, target folder when relevant, video title, action, and sanitized error message.
- [x] The UI distinguishes retryable failures from blocked account/session/risk-control failures.
- [x] Successful remote changes are visible in the local library view after execution.
- [x] Failed/skipped/blocked items remain visible after app restart.
- [x] The UI does not display raw cookies, CSRF tokens, callback URLs, or account identifiers.
- [x] Tests or build-time checks cover the typed result contract and status rendering paths.
- [x] `pnpm build` passes.

## Evidence

- Implemented by commit `6e3e8dc` (`feat: surface Bilibili favorite operation history`).
- Verified by `pnpm build` on 2026-06-11; full `cargo test --manifest-path src-tauri/Cargo.toml` also passed with 77 tests.
- Final report: `docs/research/bilibili-favorites-remote-operations-report.html`.

## Blocked by

- Issue 3: Execute favorite move plans
- Issue 4: Execute favorite delete plans
