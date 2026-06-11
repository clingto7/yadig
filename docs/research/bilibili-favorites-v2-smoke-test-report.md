# Bilibili Favorites V2 Smoke Test Report

Date: 2026-06-11

Links:

- PRD: `docs/prd/PRD-bilibili-favorites-v2-llm-operations.md`
- Issues: `docs/issues/bilibili-favorites-v2-llm-operations/`

## Status

This report is a pre-flight smoke-test checklist and local verification record. Real remote Bilibili writes have not been executed in this pass.

## Local Verification

- [x] TypeScript compilation passed: `pnpm exec tsc --noEmit --project tsconfig.app.json --pretty false`
- [x] Rust tests passed: `cargo test --manifest-path src-tauri/Cargo.toml`
- [x] Production build passed: `pnpm build`
- [x] Favorite draft prefill keeps LLM suggestions advisory and uses normal operation-plan execution paths.
- [x] Operation-plan item metadata stores only non-sensitive classification draft fields.
- [x] UI messages and persisted operation errors use existing redaction helpers for cookies, callback URLs, account identifiers, and LLM API keys.

## Remote Smoke-Test Preconditions

- [ ] Use only disposable Bilibili favorite folders.
- [ ] Use only non-important public test videos.
- [ ] Confirm the account is logged in with a complete write-capable session.
- [ ] Confirm LLM provider config works without recording or exporting the API key.
- [ ] Confirm logs, UI messages, operation history, and this report contain no raw cookies, CSRF tokens, callback URLs, account identifiers, or LLM API keys.

## Remote Smoke-Test Checklist

- [ ] Test LLM provider connection and record only provider/model/status.
- [ ] Run LLM classification on a small disposable favorite selection or record a sanitized provider failure.
- [ ] Create a disposable favorite folder from Workstation and confirm remote visibility on Bilibili Web.
- [ ] Rename the disposable favorite folder and confirm remote visibility on Bilibili Web.
- [ ] Copy one or two test videos between disposable folders and confirm source and target membership.
- [ ] Delete an empty disposable favorite folder and confirm remote state.
- [ ] Delete a non-empty disposable favorite folder containing only test videos and confirm remote state.
- [ ] Verify operation history for copy, create-folder, rename-folder, and delete-folder operations.

## Remote Result Log

No remote result has been recorded yet.

When this checklist is executed, record only sanitized facts:

- operation type
- disposable folder display names
- public video ids if needed
- HTTP/API result category such as `code 0 / OK` or sanitized failure kind
- local operation-plan status counts

Do not record raw request headers, cookies, CSRF values, QR callback URLs, account ids, LLM API keys, raw prompts, or full raw provider responses.
