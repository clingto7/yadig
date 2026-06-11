# Issue 6: Rename favorite folders safely

## Parent

PRD: `docs/prd/PRD-bilibili-favorites-v2-llm-operations.md`

## What to build

Allow the user to rename an existing mutable Bilibili favorite folder from Workstation. Renaming should preserve synchronized folder metadata such as introduction, privacy, and cover fields when the remote edit endpoint requires them.

If required metadata is missing or stale, the app should ask the user to resync instead of guessing defaults. Unsupported system/default/non-owned folders must be blocked before execution.

## Acceptance criteria

- [ ] Workstation exposes rename for synced mutable favorite folders.
- [ ] Blank, unchanged, or invalid new titles are rejected before execution.
- [ ] System, default, non-owned, or otherwise unsupported folders are blocked before execution when detectable from metadata.
- [ ] Rename preview shows old title, new title, folder id, and stale-metadata warning when relevant.
- [ ] Rename requires explicit confirmation after preview.
- [ ] The backend sends the Bilibili favorite folder edit request with title plus preserved introduction, privacy, and cover metadata required by the endpoint.
- [ ] Successful rename updates local collection title and preserves raw metadata.
- [ ] Rename appears in operation history with before/after title, status, and sanitized errors.
- [ ] Failed or blocked rename leaves local title unchanged unless a later sync proves the remote title changed.
- [ ] Rust tests cover edit form construction, metadata preservation, mutation eligibility, stale metadata blocking, and sanitized errors.
- [ ] TypeScript compilation, Rust tests, and production build pass.
- [ ] Manual smoke test renames a disposable folder and confirms it on Bilibili Web.

## Blocked by

- Issue 5: Create favorite folders from Workstation
