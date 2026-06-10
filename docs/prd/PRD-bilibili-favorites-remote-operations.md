# PRD: Bilibili Favorites Remote Operations MVP

## Problem Statement

yadig 已经可以同步 Bilibili 收藏、关注和稍后再看，并能用本地 metadata/LLM 给资源打标签和生成音频提取计划。但当前整理行为只停留在本地：用户可以看到建议，却不能把结果真正应用回 Bilibili 账号。

作为个人媒体工作站用户，我需要把 B 站收藏夹里的视频按自己的整理意图移动到目标收藏夹，或者从某个收藏夹删除不再需要的收藏内容。这个能力必须低风险、可预览、可确认、可追踪，因为它会修改用户真实 Bilibili 账号状态。

## Solution

第一期只做 Bilibili 收藏夹视频的远端移动和远端删除。yadig 同步当前账号的收藏夹列表、收藏夹内容和视频所属收藏夹关系，在 Workstation 中展示可筛选的收藏内容。用户选择若干视频后，可以生成一个操作计划：

- 将这些视频从源收藏夹移动到目标收藏夹
- 从源收藏夹删除这些视频的收藏记录

执行前必须展示操作预览，包括源收藏夹、目标收藏夹、视频数量、视频标题、不可执行原因和风险提示。用户显式确认后，后端按小批次、限速调用 Bilibili Web API 执行，并记录每条操作的成功、失败、错误信息和可重试状态。

本 PRD 不做自动远端执行。LLM 和规则分类只能生成建议或帮助筛选，不能绕过用户确认直接修改远端账号。

## User Stories

1. As a yadig user, I want to see all my Bilibili favorite folders, so that I know what remote collections can be managed.
2. As a yadig user, I want to sync videos from each favorite folder, so that yadig has an up-to-date local snapshot before planning changes.
3. As a yadig user, I want each favorite video to show its source folder, so that I know where an operation will happen.
4. As a yadig user, I want to filter favorite videos by folder, title, UP author, Bilibili category, tags, and LLM suggestions, so that I can find items to reorganize quickly.
5. As a yadig user, I want to select multiple favorite videos, so that I can prepare a batch operation instead of editing one item at a time.
6. As a yadig user, I want to move selected videos from one favorite folder to another, so that my remote Bilibili favorites match my actual organization.
7. As a yadig user, I want to delete selected videos from a favorite folder, so that I can remove stale or unwanted favorites from my account.
8. As a yadig user, I want move and delete actions to be represented as draft operation plans first, so that nothing changes remotely until I approve it.
9. As a yadig user, I want the preview to show source folder, target folder, item count, and affected titles, so that I can catch mistakes before execution.
10. As a yadig user, I want destructive delete operations to require explicit confirmation, so that I do not accidentally remove favorites.
11. As a yadig user, I want yadig to refuse write operations when the Bilibili session lacks `bili_jct` or `DedeUserID`, so that failed or unsafe requests do not run.
12. As a yadig user, I want the app to explain why a plan cannot execute, so that I know whether I need to log in again, import a full Cookie, or resync.
13. As a yadig user, I want execution to run in small batches with pauses, so that the app avoids high-frequency account automation behavior.
14. As a yadig user, I want execution to stop on auth, CSRF, rate-limit, captcha, or risk-control errors, so that the app does not keep sending bad requests.
15. As a yadig user, I want per-item execution results, so that I know which videos succeeded, failed, or need retry.
16. As a yadig user, I want successful remote changes to update the local library snapshot, so that the UI reflects the account state after execution.
17. As a yadig user, I want failed operations to remain visible with error details, so that I can retry or handle them manually.
18. As a yadig user, I want LLM-suggested tags to help me choose items, so that metadata analysis improves remote organization without taking control away from me.
19. As a yadig user, I want operation plans to be saved locally, so that I can audit what was planned and what was executed.
20. As a yadig user, I want the app to avoid logging raw cookies, CSRF tokens, callback URLs, or account identifiers, so that debugging output does not leak credentials.

## Implementation Decisions

### Scope

The MVP supports only remote operations on Bilibili favorite folder video resources:

- Read favorite folders.
- Read favorite folder contents.
- Persist folder membership in the local media library snapshot.
- Create draft plans for move and delete.
- Execute confirmed move and delete plans.
- Record execution results.

The MVP intentionally excludes follow groups, watch-later mutation, subscriptions, bangumi follows, favorite folder creation/editing/deletion, copy operations, and automatic LLM execution.

### Remote API Contract

The implementation uses Bilibili Web endpoints identified in the batch operations research report:

- Favorite folder list: `GET /x/v3/fav/folder/created/list-all`
- Favorite folder contents: `GET /x/v3/fav/resource/list`
- Favorite resource move: `POST /x/v3/fav/resource/move`
- Favorite resource delete: `POST /x/v3/fav/resource/batch-del`

All write calls require a complete authenticated session with `SESSDATA`, `bili_jct`, and `DedeUserID`. `bili_jct` is sent as `csrf`; `DedeUserID` is used as the current account `mid` where required.

Only video resources are supported in the first implementation. Resource IDs must use Bilibili's favorite resource format for video items, not only BV IDs, because write endpoints expect resource id/type pairs.

### Domain Model

The local library already has generic item, collection, item-collection, tag, analysis, and operation plan tables. The MVP should deepen this model rather than create a disconnected favorites-only store.

Required model concepts:

- A Bilibili favorite folder is a local library collection.
- A Bilibili favorite video is a local library item.
- Folder membership is a local item-collection relation.
- Raw Bilibili metadata must preserve the resource id, resource type, BV ID, source folder id, and any favorite timestamp needed for display or write operations.
- A remote operation plan contains plan kind, source collection, target collection where relevant, action, item references, execution status, and error details.

If the existing operation plan schema is too shallow to represent source/target collections and per-item remote resource identity safely, add a focused migration rather than overloading string fields ambiguously.

### Backend Modules

The backend should introduce clear service boundaries:

- Bilibili account client: shared request builder for authenticated GET and POST form calls, cookie injection, CSRF injection, Referer/User-Agent, response parsing, and sanitized errors.
- Favorites service: favorite folder list, folder item pagination, video normalization, move execution, and delete execution.
- Batch plan builder: validates selected items, checks source/target folders, builds draft plans, and rejects impossible operations.
- Batch executor: chunks requests, rate-limits execution, classifies failures, stops on account/security failures, and returns per-item results.

These should be testable without Tauri UI code. Tauri commands should be thin wrappers around the services.

### Frontend Experience

The Workstation should gain a focused favorites management surface:

- A folder selector or filter for synced Bilibili favorite folders.
- A resource table/list with selectable videos.
- Metadata and LLM tags visible as filtering aids.
- Actions for "Move to folder" and "Delete from folder".
- A plan preview panel before execution.
- A result panel after execution showing succeeded, failed, skipped, and retryable items.

The UI must avoid implying that LLM suggestions have already changed the remote account. Suggested tags and actions remain local until the user creates and executes a plan.

### Safety Rules

- No remote write operation runs without an explicit user confirmation.
- Delete is treated as destructive and requires stronger confirmation text than move.
- Plans must validate that every item still has a known source folder and remote resource identity.
- Moving an item to the same folder is skipped before execution.
- Operations require a complete Bilibili session; SESSDATA-only login is read-only for this feature.
- Execution uses small batches and pauses between batches.
- Auth, CSRF, captcha, rate-limit, or risk-control responses stop the remaining execution immediately.
- Logs and errors must not include raw cookies, bearer tokens, callback URLs, CSRF values, or account identifiers.

### Data Freshness

Before executing a plan, yadig should know whether the local snapshot is fresh enough. The MVP can use a simple rule: warn if the relevant folder has not been synced during the current app session or if the stored `last_synced_at` is older than a conservative threshold.

The executor should handle stale plans gracefully. If Bilibili reports that a resource no longer exists in the source folder, mark that item as skipped or failed with a clear stale-state message rather than treating the whole execution as successful.

### Execution Result Semantics

Each plan item can end in one of these user-visible states:

- Pending: planned but not executed.
- Running: currently executing.
- Success: remote API accepted the operation and local state was updated.
- Skipped: no-op or stale item that should not be retried as-is.
- Failed: operation did not complete and may need user action.
- Blocked: execution stopped because account/session/risk-control state makes further calls unsafe.

## Testing Decisions

Good tests should verify observable behavior and safety rules, not private implementation details. The most important behavior is that yadig only constructs valid plans, refuses unsafe execution, maps Bilibili API failures into useful statuses, and never logs secrets.

Modules to test:

- Session/write eligibility: complete session is required for move/delete; SESSDATA-only session is rejected.
- Favorite folder normalization: Bilibili folder and resource JSON becomes local collection/item/membership data with the remote resource identity preserved.
- Plan builder: move and delete plans contain the correct action, source folder, target folder, and selected items; same-folder moves are skipped.
- Executor: chunks requests, stops on auth/CSRF/risk-control failures, records per-item status, and updates local state only after success.
- Error redaction: raw cookies, csrf tokens, callback URLs, and account IDs do not appear in logs or returned errors.
- Frontend build: typed Tauri command contracts compile and the Workstation actions cannot call execution without a plan.

Prior art:

- Existing Rust unit tests cover Bilibili auth/session serialization, URL parsing, WBI signing, audio selection, and media workstation operation plan creation.
- Existing frontend build is the primary guard for TypeScript/Tauri contract drift.
- The QR poll log redaction test is a model for adding write-operation error redaction tests.

## Out of Scope

- Managing Bilibili follow groups.
- Managing watch-later remote state.
- Managing subscriptions, bangumi follows, channels, playlists, or UP dynamic subscriptions.
- Creating, renaming, deleting, or privacy-editing favorite folders.
- Copying favorites between folders.
- Automatically executing LLM recommendations without user confirmation.
- Browser automation, Playwright-driven UI clicking, or browser extension approaches.
- Circumventing Bilibili captcha, risk control, account limits, or access restrictions.
- Replacing Tauri Store with OS keychain storage. This remains a future security improvement.

## Further Notes

This feature depends on the recently completed Bilibili login persistence work. The app now restores a full `BiliSession` across restarts and can preserve the CSRF data needed for write endpoints when the user logs in through QR code or imports a full Cookie.

Because these Bilibili APIs are not official product contracts, runtime behavior must be verified with a small test folder and a small number of videos before broad use. The first manual smoke test should use a disposable favorite folder, move one or two test videos into another disposable folder, then delete one test video from the disposable folder and confirm the change on Bilibili Web.

The implementation should keep the user-facing mental model simple: sync account state, select resources, preview plan, confirm, execute slowly, review results.
