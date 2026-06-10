# Issue 3: Execute favorite move plans

## Parent

PRD: `docs/prd/PRD-bilibili-favorites-remote-operations.md`

## What to build

Execute confirmed Bilibili favorite move plans against the user's real Bilibili account. The executor should call the favorite resource move endpoint with the selected items' source folder, target folder, account mid, resource id/type pairs, and CSRF token. Execution must be small-batch and low-frequency, stop on account/security failures, and return per-item results.

This is a human-in-the-loop slice because it must be smoke-tested with disposable Bilibili favorite folders and a small number of test videos.

## Acceptance criteria

- [x] Move execution requires an explicit user confirmation after plan preview.
- [x] Move execution refuses sessions that lack `SESSDATA`, `bili_jct`, or `DedeUserID`.
- [x] The backend sends Bilibili favorite move requests with `csrf=bili_jct` and resource id/type pairs.
- [x] Execution chunks requests into small batches and pauses between batches.
- [x] Auth, CSRF, captcha, rate-limit, risk-control, or malformed-plan responses stop remaining execution.
- [x] Each item receives a result status: success, skipped, failed, or blocked.
- [x] Successful moves update local folder membership after remote success.
- [x] Failed or blocked items keep an error message that is useful but does not expose cookies, CSRF tokens, callback URLs, or account identifiers.
- [x] Tests cover session eligibility, request planning, stop-on-security-failure behavior, and secret redaction.
- [x] Manual smoke test moves one or two videos between disposable favorite folders and confirms the result on Bilibili Web.
- [x] `cargo test --manifest-path src-tauri/Cargo.toml` and `pnpm build` pass.

## Evidence

- Implemented by commit `dfaa4dd` (`feat: execute Bilibili favorite move plans`).
- Verified by `cargo test --manifest-path src-tauri/Cargo.toml` and `pnpm build` on 2026-06-11.
- Real remote smoke test moved `BV13U7k6HEjK` from disposable folder `ydg-src-0611` to disposable folder `ydg-tgt-0611` and confirmed source/target state on Bilibili Web API with sanitized `HTTP 200 / code 0 / OK` response.
- Final report: `docs/research/bilibili-favorites-remote-operations-report.html`.

## Blocked by

- Issue 1: Sync Bilibili favorite folders and membership
- Issue 2: Create safe favorite operation plans
