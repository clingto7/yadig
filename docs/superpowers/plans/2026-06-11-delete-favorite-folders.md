# Delete Favorite Folders Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add safe deletion of Bilibili favorite folders from Workstation, including non-empty folders.

**Architecture:** Reuse the existing operation-plan model with a new folder-delete kind and one plan item per deleted folder. The backend owns mutation eligibility, typed confirmation, and Bilibili `/x/v3/fav/folder/del` request construction; the frontend owns impact preview, typed confirmation input, local collection cleanup after success, and operation history display.

**Tech Stack:** Rust/Tauri commands, existing Bilibili client, SQLite via `@tauri-apps/plugin-sql`, React/TypeScript Workstation UI.

---

### Task 1: Backend Plan And Client Contract

**Files:**
- Modify: `src-tauri/src/library.rs`
- Modify: `src-tauri/src/bili/client.rs`

- [x] Write failing Rust tests for folder-delete plan metadata and delete form construction.
- [x] Run targeted Rust tests and confirm they fail because the new kind/request helpers do not exist.
- [x] Add `BiliFavoriteFolderDelete`, `FavoriteFolderDeletePlanRequest`, impact metadata, and mutation blocking.
- [x] Add `FavoriteFolderDeleteRequest`, form builder, and client method for `/x/v3/fav/folder/del`.
- [x] Run targeted Rust tests and confirm they pass.

### Task 2: Backend Execution Command

**Files:**
- Modify: `src-tauri/src/commands/library.rs`
- Modify: `src-tauri/src/lib.rs`

- [x] Write failing Rust tests for exact `DELETE <folder name>` confirmation, success, and blocked errors.
- [x] Add create/execute Tauri commands and runner-based execution helper.
- [x] Register commands in `src-tauri/src/lib.rs`.
- [x] Run targeted Rust tests and confirm they pass.

### Task 3: Frontend And Local Cleanup

**Files:**
- Modify: `src/lib/tauri.ts`
- Modify: `src/lib/db.ts`
- Create: `src/lib/favorite-folder-delete-operation.contract.ts`
- Modify: `src/pages/workstation-page.tsx`

- [x] Add TypeScript contracts for folder-delete plan and execution.
- [x] Add local cleanup helper that deletes the successful folder collection and its memberships only.
- [x] Add Workstation folder selector, impact preview, typed confirmation, execution button, and history rendering.
- [x] Ensure non-empty folder deletion is allowed and item titles are visible before confirmation.
- [x] Run TypeScript compilation.

### Task 4: Verification And Commit

**Files:**
- Modify: `docs/issues/bilibili-favorites-v2-llm-operations/007-delete-favorite-folders-with-strong-confirmation.md`

- [x] Run `cargo test --manifest-path src-tauri/Cargo.toml`.
- [x] Run `pnpm exec tsc --noEmit --project tsconfig.app.json --pretty false`.
- [x] Run `pnpm build`.
- [x] Run `git diff --check`.
- [x] Run the repository secret scan.
- [x] Mark automated acceptance criteria complete; keep live Bilibili smoke tests unchecked unless actually performed.
- [x] Commit as `feat: delete favorite folders from workstation`.
