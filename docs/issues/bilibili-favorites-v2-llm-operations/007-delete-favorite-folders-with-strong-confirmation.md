# Issue 7: Delete favorite folders with strong confirmation

## Parent

PRD: `docs/prd/PRD-bilibili-favorites-v2-llm-operations.md`

## What to build

Allow the user to delete a Bilibili favorite folder from Workstation, including non-empty folders. This is the highest-risk operation in this PRD and must require an impact preview plus exact typed confirmation.

Successful deletion should remove the local collection and memberships while preserving library items that still belong to other local contexts. Failed or blocked deletion should leave local state intact.

## Acceptance criteria

- [ ] Workstation exposes delete for synced mutable favorite folders.
- [ ] System, default, non-owned, or otherwise unsupported folders are blocked before execution when detectable from metadata.
- [ ] Delete preview shows folder title, folder id, known item count, snapshot freshness, and known item titles for non-empty folders.
- [ ] Non-empty folder deletion is allowed.
- [ ] Delete requires exact confirmation text `DELETE <folder name>`.
- [ ] Folder deletion requires a complete Bilibili write session.
- [ ] The backend sends the Bilibili favorite folder delete request with folder id and CSRF token.
- [ ] Auth, CSRF, captcha, rate-limit, risk-control, and malformed-plan failures are classified as blocked and do not retry automatically.
- [ ] Successful deletion removes the local collection and its memberships.
- [ ] Successful deletion does not unconditionally delete library items that may still be present in another folder or resource context.
- [ ] Delete appears in operation history with impact summary, status, and sanitized errors.
- [ ] Failed or blocked deletion leaves local collection and memberships intact.
- [ ] Rust tests cover delete form construction, typed confirmation, non-empty impact metadata, mutation eligibility, local cleanup semantics, and secret redaction.
- [ ] TypeScript compilation, Rust tests, and production build pass.
- [ ] Manual smoke test deletes an empty disposable folder.
- [ ] Manual smoke test deletes a non-empty disposable folder with test videos and confirms the remote state on Bilibili Web.

## Blocked by

- Issue 5: Create favorite folders from Workstation
