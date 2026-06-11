# Issue 8: Prefill drafts from LLM suggestions

## Parent

PRD: `docs/prd/PRD-bilibili-favorites-v2-llm-operations.md`

## What to build

Connect classification review to operation-plan creation. After filtering and selecting classification results, the user should be able to prefill a copy, move, or delete draft from LLM-suggested actions and target folders, then override any action or target before creating the actual draft.

This slice completes the intended human-in-the-loop workflow: LLM suggests, the user filters and selects, yadig creates a normal previewable draft, and only the user can confirm execution.

## Acceptance criteria

- [x] Workstation can group selected classification results by suggested action.
- [x] Users can choose to use LLM-suggested action, force a single action, or remove items before draft creation.
- [x] Users can choose a matched existing target folder or create a target folder through the folder-create flow before creating a copy/move draft.
- [x] Suggested target folder names are never used for remote writes without user review.
- [x] Draft preview identifies items that came from classification results and shows their category, confidence, and suggestion provenance.
- [x] Users can override action and target folder before creating the operation plan.
- [x] Generated copy/move/delete drafts use the same validation rules and execution paths as manually created drafts.
- [x] LLM suggestions cannot trigger remote execution directly.
- [x] Operation history records the selected action and normal per-item status, without storing API keys, raw prompts, cookies, or full raw provider responses.
- [x] Frontend tests or contract checks cover suggestion grouping, override behavior, and no-direct-execution behavior.
- [x] TypeScript compilation, Rust tests, and production build pass.

## Implementation notes

- Added `src/lib/classification-draft-prefill.ts` for selected-result grouping, action override, target-folder matching, and non-sensitive draft metadata.
- Workstation now exposes a `Classification Draft Prefill` flow that creates normal copy/move/delete operation plans only; execution still requires the existing explicit confirmation buttons.
- Existing target-folder suggestions can be applied only by selecting a matched local folder. Unmatched suggested target names can prefill the folder-create title, but do not write remotely.
- Draft preview and operation history display classification category, confidence, provenance, suggested action, and suggested target.
- Local verification on 2026-06-11:
  - `pnpm exec tsc --noEmit --project tsconfig.app.json --pretty false`
  - `cargo test --manifest-path src-tauri/Cargo.toml`
  - `pnpm build`

## Blocked by

- Issue 3: Filter and select classification results
- Issue 4: Copy favorite items through operation plans
- Issue 5: Create favorite folders from Workstation
