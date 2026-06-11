# Issue 3: Filter and select classification results

## Parent

PRD: `docs/prd/PRD-bilibili-favorites-v2-llm-operations.md`

## What to build

Turn persisted classification results into a review surface in Workstation. The user should be able to filter Bilibili favorite items by source folder, category, tags, confidence threshold, suggested action, suggested target folder, title, author, and Bilibili category, then select all filtered items or manually adjust the selection.

This slice does not execute remote operations. It creates a reliable human review step that feeds later copy/move/delete draft creation.

## Acceptance criteria

- [ ] Workstation shows category, tags, confidence, suggested action, suggested target folder, reason, and provenance for favorite items with classifications.
- [ ] Users can filter by source favorite folder, category, tag, confidence threshold, suggested action, suggested target folder, title, author, and Bilibili category.
- [ ] Users can select all currently filtered favorite items.
- [ ] Users can deselect individual items after bulk selection.
- [ ] Selection remains scoped to Bilibili favorite items and does not accidentally include follows or watch-later items.
- [ ] The UI distinguishes suggestions from draft plans and executed operations.
- [ ] Filter state changes do not modify remote Bilibili state.
- [ ] Pure filtering/selection behavior is covered by targeted frontend helper tests or contract checks.
- [ ] TypeScript compilation passes.

## Blocked by

- Issue 2: Classify favorite items with persisted LLM results
