# Issue 5: Create favorite folders from Workstation

## Parent

PRD: `docs/prd/PRD-bilibili-favorites-v2-llm-operations.md`

## What to build

Allow the user to create a Bilibili favorite folder from Workstation. The created folder should be available immediately as a copy/move target after the remote call succeeds and local collections refresh.

This slice establishes folder operation plans and history for non-resource operations. It should not require LLM classification to be present, but it should support a later flow where an LLM-suggested target folder can be created after user review.

## Acceptance criteria

- [x] Workstation exposes a create-folder action with title, optional introduction, and privacy setting.
- [x] Blank or invalid titles are rejected before execution.
- [x] Folder creation requires a complete Bilibili write session.
- [x] The backend sends the Bilibili favorite folder add request with title, introduction, privacy, and CSRF token.
- [x] User sees a preview before execution and must explicitly confirm.
- [x] Successful creation refreshes local favorite-folder collections and makes the new folder available as a copy/move target.
- [x] Folder creation appears in operation history with status, title, remote folder id when known, and sanitized errors.
- [x] Failed or blocked creation does not create a misleading local collection.
- [x] Rust tests cover create-folder form construction, write-session requirement, sanitized errors, and plan/history serialization.
- [x] TypeScript compilation, Rust tests, and production build pass.
- [ ] Manual smoke test creates a disposable folder and confirms it on Bilibili Web.

## Blocked by

None - can start immediately.
