# Issue 2: Classify favorite items with persisted LLM results

## Parent

PRD: `docs/prd/PRD-bilibili-favorites-v2-llm-operations.md`

## What to build

Build the explicit LLM classification workflow for Bilibili favorite items. A user should be able to choose a synced favorite folder or selected favorite items, run LLM classification in bounded chunks, and persist structured results locally with provenance.

The result contract must support category, tags, reason, confidence, suggested action, and suggested target folder. LLM output is only advisory. It must not create, move, copy, delete, or rename anything on Bilibili.

If an LLM chunk fails, the failure remains visible and successful chunks remain available. The app must not silently substitute local metadata fallback and label it as LLM output.

## Acceptance criteria

- [ ] Workstation can run explicit LLM classification for a selected favorite folder or selected favorite items.
- [ ] Favorite item payloads include only classification metadata, not Bilibili cookies, CSRF tokens, LLM API keys, callback URLs, or account identifiers.
- [ ] Requests are chunked with a conservative fixed chunk size for the first version.
- [ ] Each valid result persists item id, category, tags, reason, confidence, suggested action, suggested target folder, provenance, provider/model, and analysis timestamp.
- [ ] Result validation rejects or normalizes unknown item ids, duplicate results, invalid confidence values, unsupported actions, malformed target suggestions, and malformed JSON.
- [ ] Partial chunk success is saved; failed chunks are reported with sanitized errors.
- [ ] Explicit local metadata classification remains available and stores provenance as local metadata, not LLM.
- [ ] The Workstation displays classification provenance so users can tell LLM output from local metadata output.
- [ ] Rust tests cover parsing, validation, chunk handling, strict LLM failure semantics, and local metadata provenance.
- [ ] Database/filtering helper tests or contract checks cover persistence of category, confidence, suggested action, target suggestion, and timestamp.
- [ ] TypeScript compilation and Rust tests pass.

## Blocked by

- Issue 1: Test and store LLM provider configuration
