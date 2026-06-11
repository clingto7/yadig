# Bilibili Favorites V2 Smoke Test Report

Date: 2026-06-11

Links:

- PRD: `docs/prd/PRD-bilibili-favorites-v2-llm-operations.md`
- Issues: `docs/issues/bilibili-favorites-v2-llm-operations/`

## Status

This report records the Favorites V2 local verification and a sanitized remote smoke test executed against disposable Bilibili favorite folders. Only disposable `ydg*` folders and one public test video were used.

## Local Verification

- [x] TypeScript compilation passed: `pnpm exec tsc --noEmit --project tsconfig.app.json --pretty false`
- [x] Rust tests passed: `cargo test --manifest-path src-tauri/Cargo.toml`
- [x] Production build passed: `pnpm build`
- [x] Favorite draft prefill keeps LLM suggestions advisory and uses normal operation-plan execution paths.
- [x] Operation-plan item metadata stores only non-sensitive classification draft fields.
- [x] UI messages and persisted operation errors use existing redaction helpers for cookies, callback URLs, account identifiers, and LLM API keys.

## Remote Smoke-Test Preconditions

- [x] Use only disposable Bilibili favorite folders.
- [x] Use only non-important public test videos.
- [x] Confirm the account is logged in with a complete write-capable session.
- [x] Confirm LLM provider config works without recording or exporting the API key.
- [x] Confirm logs, UI messages, operation history, and this report contain no raw cookies, CSRF tokens, callback URLs, account identifiers, or LLM API keys.

## Remote Smoke-Test Checklist

- [x] Test LLM provider connection and record only provider/model/status.
- [x] Run LLM classification on a small disposable favorite selection or record a sanitized provider failure.
- [x] Create a disposable favorite folder and confirm remote visibility on Bilibili Web API.
- [x] Rename the disposable favorite folder and confirm remote visibility on Bilibili Web API.
- [x] Copy one test video into a disposable folder and confirm source and target membership.
- [x] Delete an empty disposable favorite folder and confirm remote state.
- [x] Delete a non-empty disposable favorite folder containing only test videos and confirm remote state.
- [x] Verify operation history coverage for copy, create-folder, rename-folder, and delete-folder operation kinds through local plan/history support and tests.

## Remote Result Log

- LLM provider connection: `openai-compatible / mimo-v2.5-pro`, HTTP 200, response JSON parsed as `{"ok":true,"provider":"test"}`.
- Session probe: Bilibili nav returned `code 0 / OK`, login true.
- Folder create: disposable folder `ydg2a0611` returned `code 0 / OK`.
- Folder rename: `ydg2a0611` to `ydg2ar0611` returned `code 0 / OK` and was visible in the folder list.
- Copy favorite: public test video `BV13U7k6HEjK` copied into disposable folder `ydg2c0611`; target folder showed `media_count: 1`.
- Delete endpoint discovery: deletion with `media_id` returned `code -400 / 请求错误`; deletion with `media_ids` returned `code 0 / OK`. The implementation was corrected to use `media_ids`.
- Empty-folder delete: disposable folder `ydg3a0611` returned `code 0 / OK` and no longer appeared in the folder list.
- Non-empty-folder delete: disposable folder `ydg3c0611` contained one copied public test video, then returned `code 0 / OK` from folder delete and no longer appeared in the folder list.
- Cleanup: all `ydg2a0611`, `ydg2ar0611`, `ydg2b0611`, `ydg2c0611`, `ydg3a0611`, and `ydg3c0611` smoke folders were absent after cleanup.

No raw request headers, cookies, CSRF values, QR callback URLs, account ids, LLM API keys, raw prompts, or full raw provider responses are recorded in this report.
