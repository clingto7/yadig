# Issue 8: Prefill drafts from LLM suggestions

## Parent

PRD: `docs/prd/PRD-bilibili-favorites-v2-llm-operations.md`

## What to build

Connect classification review to operation-plan creation. After filtering and selecting classification results, the user should be able to prefill a copy, move, or delete draft from LLM-suggested actions and target folders, then override any action or target before creating the actual draft.

This slice completes the intended human-in-the-loop workflow: LLM suggests, the user filters and selects, yadig creates a normal previewable draft, and only the user can confirm execution.

## Acceptance criteria

- [ ] Workstation can group selected classification results by suggested action.
- [ ] Users can choose to use LLM-suggested action, force a single action, or remove items before draft creation.
- [ ] Users can choose a matched existing target folder or create a target folder through the folder-create flow before creating a copy/move draft.
- [ ] Suggested target folder names are never used for remote writes without user review.
- [ ] Draft preview identifies items that came from classification results and shows their category, confidence, and suggestion provenance.
- [ ] Users can override action and target folder before creating the operation plan.
- [ ] Generated copy/move/delete drafts use the same validation rules and execution paths as manually created drafts.
- [ ] LLM suggestions cannot trigger remote execution directly.
- [ ] Operation history records the selected action and normal per-item status, without storing API keys, raw prompts, cookies, or full raw provider responses.
- [ ] Frontend tests or contract checks cover suggestion grouping, override behavior, and no-direct-execution behavior.
- [ ] TypeScript compilation, Rust tests, and production build pass.

## Blocked by

- Issue 3: Filter and select classification results
- Issue 4: Copy favorite items through operation plans
- Issue 5: Create favorite folders from Workstation
