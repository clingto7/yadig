# Issue 9: Favorites V2 smoke test and final report

## Parent

PRD: `docs/prd/PRD-bilibili-favorites-v2-llm-operations.md`

## What to build

Run an end-to-end validation pass for Favorites V2 and record the result in a sanitized research report. The smoke test should use disposable Bilibili favorite folders and a small number of non-important public test videos.

The report should cover LLM configuration testing, LLM classification provenance, copy favorite behavior, folder create, folder rename, empty-folder delete, non-empty-folder delete, execution history, and redaction behavior.

## Acceptance criteria

- [x] A manual smoke-test checklist exists before remote writes are executed.
- [ ] Smoke test uses disposable folders and non-important public test videos only.
- [ ] Smoke test verifies LLM provider connection without recording the API key.
- [ ] Smoke test verifies LLM classification output or records a sanitized provider failure.
- [ ] Smoke test creates a disposable folder and confirms remote visibility.
- [ ] Smoke test renames a disposable folder and confirms remote visibility.
- [ ] Smoke test copies one or two test videos and confirms source and target membership.
- [ ] Smoke test deletes an empty disposable folder and confirms remote state.
- [ ] Smoke test deletes a non-empty disposable folder with test videos and confirms remote state.
- [ ] Smoke test verifies operation history for copy and folder operations.
- [ ] Smoke test verifies raw Bilibili cookies, CSRF tokens, callback URLs, account identifiers, and LLM API keys do not appear in UI messages, operation history, logs, or report text.
- [x] Final report is written under `docs/research/` and links back to the PRD and issue directory.
- [x] `cargo test --manifest-path src-tauri/Cargo.toml`, `pnpm exec tsc --noEmit --project tsconfig.app.json --pretty false`, and `pnpm build` pass on the final implementation.

## Notes

- Pre-flight checklist and local verification report: `docs/research/bilibili-favorites-v2-smoke-test-report.md`.
- Real remote Bilibili write checks remain unchecked until they are executed with disposable folders and test videos.

## Blocked by

- Issue 1: Test and store LLM provider configuration
- Issue 2: Classify favorite items with persisted LLM results
- Issue 3: Filter and select classification results
- Issue 4: Copy favorite items through operation plans
- Issue 5: Create favorite folders from Workstation
- Issue 6: Rename favorite folders safely
- Issue 7: Delete favorite folders with strong confirmation
- Issue 8: Prefill drafts from LLM suggestions
