# Issue 2: Create safe favorite operation plans

## Parent

PRD: `docs/prd/PRD-bilibili-favorites-remote-operations.md`

## What to build

Let the user select synced Bilibili favorite videos and create draft operation plans for remote move or delete actions. This slice does not execute any remote write operation. It validates that each selected item has a known source folder and remote resource identity, validates that move operations have a valid target folder, rejects or skips same-folder moves, and shows a plan preview before execution is possible.

The plan must be specific enough for later executors to run safely: action, source folder, target folder for moves, affected resource ids/types, display titles, and per-item pending status.

## Acceptance criteria

- [x] Workstation lets the user select one or more synced Bilibili favorite videos.
- [x] The user can choose "move to folder" and select a target favorite folder.
- [x] The user can choose "delete from folder" for selected videos.
- [x] Plan creation refuses execution-unsafe items that lack source folder id, remote resource id, or resource type.
- [x] Move plans skip or reject items whose source folder equals the target folder.
- [x] Delete plans require a source folder for every selected item.
- [x] Draft plans are saved locally with action, source folder, target folder when relevant, selected items, and pending status.
- [x] The UI shows a preview containing item count, affected titles, source folder, target folder where relevant, and blocked/skipped reasons.
- [x] The UI does not call Bilibili write endpoints in this slice.
- [x] Tests cover plan validation for valid moves, same-folder moves, valid deletes, and missing remote identity.
- [x] `cargo test --manifest-path src-tauri/Cargo.toml` and `pnpm build` pass.

## Evidence

- Implemented by commit `2a441a0` (`feat: create Bilibili favorite operation drafts`).
- Verified by `cargo test --manifest-path src-tauri/Cargo.toml` and `pnpm build` on 2026-06-11.
- Final report: `docs/research/bilibili-favorites-remote-operations-report.html`.

## Blocked by

- Issue 1: Sync Bilibili favorite folders and membership
