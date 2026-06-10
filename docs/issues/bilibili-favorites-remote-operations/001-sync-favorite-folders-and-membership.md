# Issue 1: Sync Bilibili favorite folders and membership

## Parent

PRD: `docs/prd/PRD-bilibili-favorites-remote-operations.md`

## What to build

Build the read-only foundation for Bilibili favorite remote operations. After the user syncs Bilibili in Workstation, yadig should know the logged-in account's favorite folders, the videos inside those folders, and each video's folder membership. The result must be visible enough in Workstation for the user to filter Bilibili favorite videos by folder before creating any remote operation plan.

This slice should preserve the remote identity needed by later write operations: favorite folder id, video resource id, resource type, BV id, and source folder membership. It should update the local media library snapshot without changing the user's remote Bilibili account.

## Acceptance criteria

- [x] Sync fetches the current user's Bilibili favorite folder list using the authenticated session.
- [x] Sync fetches paginated video contents for each favorite folder selected by the favorites scope.
- [x] Favorite folders are persisted as local library collections.
- [x] Favorite videos are persisted as local library items.
- [x] Video-to-folder membership is persisted locally and survives app restart.
- [x] Raw metadata preserves remote resource id, resource type, BV id, source folder id, and favorite timestamp when present.
- [x] Workstation can display or filter Bilibili favorite videos by favorite folder.
- [x] No remote write endpoint is called by this slice.
- [x] Incomplete sessions return a clear read/auth error without logging cookies or tokens.
- [x] Rust tests cover folder/video normalization and membership preservation.
- [x] `cargo test --manifest-path src-tauri/Cargo.toml` and `pnpm build` pass.

## Evidence

- Implemented by commit `388c59e` (`feat: sync Bilibili favorite folders`).
- Verified by `cargo test --manifest-path src-tauri/Cargo.toml` and `pnpm build` on 2026-06-11.
- Final report: `docs/research/bilibili-favorites-remote-operations-report.html`.

## Blocked by

None - can start immediately.
