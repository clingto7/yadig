# Issue 4: Execute favorite delete plans

## Parent

PRD: `docs/prd/PRD-bilibili-favorites-remote-operations.md`

## What to build

Execute confirmed Bilibili favorite delete plans against the user's real Bilibili account. Delete removes selected video resources from a source favorite folder. Because this is destructive, it must require stronger confirmation than move, execute slowly, stop on account/security failures, and record per-item results.

This is a human-in-the-loop slice because it must be smoke-tested with disposable Bilibili favorite folders and test videos.

## Acceptance criteria

- [x] Delete execution requires explicit destructive confirmation after plan preview.
- [x] Delete execution refuses sessions that lack `SESSDATA`, `bili_jct`, or `DedeUserID`.
- [x] The backend sends Bilibili favorite batch-delete requests with `csrf=bili_jct`, source folder id, and resource id/type pairs.
- [x] Execution chunks requests into small batches and pauses between batches.
- [x] Auth, CSRF, captcha, rate-limit, risk-control, or malformed-plan responses stop remaining execution.
- [x] Each item receives a result status: success, skipped, failed, or blocked.
- [x] Successful deletes remove or update local folder membership only after remote success.
- [x] Failed or blocked items keep an error message that is useful but does not expose cookies, CSRF tokens, callback URLs, or account identifiers.
- [x] Tests cover destructive confirmation gating, session eligibility, request planning, stop-on-security-failure behavior, and secret redaction.
- [x] Manual smoke test deletes one test video from a disposable favorite folder and confirms the result on Bilibili Web.
- [x] `cargo test --manifest-path src-tauri/Cargo.toml` and `pnpm build` pass.

## Evidence

- Implemented by commit `9c20a14` (`feat: execute Bilibili favorite delete plans`).
- Verified by `cargo test --manifest-path src-tauri/Cargo.toml` and `pnpm build` on 2026-06-11.
- Real remote smoke test deleted `BV13U7k6HEjK` from disposable folder `ydg-tgt-0611` and confirmed it was absent via Bilibili Web API with sanitized `HTTP 200 / code 0 / OK` response.
- Final report: `docs/research/bilibili-favorites-remote-operations-report.html`.

## Blocked by

- Issue 1: Sync Bilibili favorite folders and membership
- Issue 2: Create safe favorite operation plans
