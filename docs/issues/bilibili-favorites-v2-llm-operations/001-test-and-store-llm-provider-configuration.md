# Issue 1: Test and store LLM provider configuration

## Parent

PRD: `docs/prd/PRD-bilibili-favorites-v2-llm-operations.md`

## What to build

Build the formal OpenAI-compatible LLM configuration path. The user should be able to enter a provider label, base URL, model, and API key in Settings, save them locally, and run a connection test that proves the configured provider can return parseable JSON through chat completions.

This slice makes LLM failures explicit. Missing keys, authentication failures, network errors, incompatible response shape, and invalid JSON output should be reported as LLM configuration/test failures instead of silently falling back to local metadata rules.

The API key is sensitive local configuration. It must not be committed, written to docs, stored in operation history, or echoed in errors.

## Acceptance criteria

- [x] Settings can store and reload LLM provider label, base URL, model, and API key from local Tauri Store.
- [x] Settings includes a "Test LLM" action that sends a minimal request through the configured OpenAI-compatible chat-completions endpoint.
- [x] The test validates HTTP success, authentication, response shape, and parseable JSON message content.
- [x] The test distinguishes missing config, auth failure, network failure, incompatible API response, and invalid JSON output in user-facing messages.
- [x] The provider client supports both JSON-response-format requests and prompt-only JSON fallback when `response_format` is not accepted.
- [x] API keys and bearer tokens are redacted from Rust errors, frontend messages, logs, and test fixtures.
- [x] No real API key is written into source, docs, SQLite, or operation history.
- [x] Existing metadata fallback remains available as a separate explicit path but is not used to mask a failed LLM test.
- [x] Rust tests cover request construction, response parsing, compatibility fallback, and secret redaction.
- [x] TypeScript compilation and Rust tests pass.

## Notes

- Implemented in Settings LLM Integration and `llm_test_provider`.
- Verified by `docs/research/bilibili-favorites-v2-smoke-test-report.md` and the 2026-06-11 verification pass.
- Secret scan over `src`, `src-tauri`, and `docs` found no committed API keys or bearer tokens.

## Blocked by

None - can start immediately.
