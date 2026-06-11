# PRD: Bilibili Favorites V2 and LLM-Assisted Operations

## Problem Statement

yadig 已经可以同步 Bilibili 收藏夹和收藏条目，创建并执行收藏条目的 move/delete 操作计划，也能保存执行历史。当前能力仍有两个明显缺口。

第一，用户只能整理已有收藏夹，不能在 yadig 中创建、重命名或删除收藏夹，也不能把条目复制到另一个收藏夹并保留原 membership。这使完整的收藏整理流程仍然依赖 Bilibili Web。

第二，现有 LLM 集成只提供基础 OpenAI-compatible 调用和简单标签结果。调用失败时会自动降级到本地 metadata fallback，用户难以判断结果到底来自真实 LLM 还是本地规则，也不能按分类、置信度或建议动作筛选结果并有选择地生成批量操作计划。

作为个人媒体工作站用户，我需要先让 LLM 对收藏条目进行可追踪的分类，再审核分类结果，选择其中一部分执行 copy、move 或 delete。同时，我需要直接管理收藏夹生命周期。所有远端修改都必须继续保持可预览、可确认、可追踪，LLM 不能直接控制真实账号。

## Solution

在现有 Bilibili media workstation 上增加 Favorites V2 工作流：

1. 用户可以创建、重命名和删除 Bilibili 收藏夹。
2. 用户可以将选中的收藏条目复制到已有或新建收藏夹，并保留源收藏夹 membership。
3. 用户可以配置并测试 OpenAI-compatible LLM provider，然后按批次分析收藏条目。
4. LLM 返回结构化 category、tags、reason、confidence、suggested action 和 suggested target folder。
5. Workstation 可以按分类结果筛选、选择或排除条目，再由用户生成 copy、move 或 delete 草稿。
6. 所有远端操作继续通过 operation plan、预览、显式确认、限速执行和本地历史完成。

第一版使用用户本机配置的 OpenAI-compatible provider。当前目标 profile 为：

- Base URL: `https://token-plan-cn.xiaomimimo.com/v1`
- Model: `mimo-v2.5-pro`
- API key: 只保存在本机设置存储中，不进入源码、文档、日志或 operation history。

## User Stories

1. As a yadig user, I want to create a Bilibili favorite folder, so that I can prepare a destination before reorganizing items.
2. As a yadig user, I want to choose a title, introduction, and privacy setting when creating a folder, so that the remote folder is created with the intended metadata.
3. As a yadig user, I want a newly created folder to appear locally without restarting the app, so that I can use it immediately.
4. As a yadig user, I want to rename an existing favorite folder, so that its title reflects my current organization.
5. As a yadig user, I want renaming to preserve the folder's existing introduction, privacy, and cover, so that a title-only change does not alter unrelated settings.
6. As a yadig user, I want system or non-owned folders to be blocked from unsupported mutations, so that yadig does not send invalid requests.
7. As a yadig user, I want to delete an empty favorite folder, so that obsolete organization can be removed.
8. As a yadig user, I want to delete a non-empty favorite folder, so that I can remove a folder and all of its memberships in one deliberate action.
9. As a yadig user, I want folder deletion preview to show the folder name and item count, so that I understand the impact.
10. As a yadig user, I want non-empty folder deletion to show known item titles, so that I can catch mistakes before confirmation.
11. As a yadig user, I want folder deletion to require the exact text `DELETE <folder name>`, so that accidental destructive clicks do not execute.
12. As a yadig user, I want successful folder deletion to remove the local collection and memberships, so that local state matches the remote account.
13. As a yadig user, I want folder create, rename, and delete attempts recorded in operation history, so that I can audit account changes.
14. As a yadig user, I want to copy selected favorite items to another folder, so that they remain in the source folder and also appear in the target folder.
15. As a yadig user, I want copy to use the same preview and confirmation model as move, so that remote effects are clear.
16. As a yadig user, I want copying to the same source folder to be skipped, so that yadig does not send a meaningless request.
17. As a yadig user, I want copying an item already present in the target folder to be treated as a no-op or stale result, so that it is not presented as a new copy.
18. As a yadig user, I want successful copy operations to add local target membership without deleting source membership, so that the snapshot reflects copy semantics.
19. As a yadig user, I want copy operations to be batched and rate-limited, so that account automation remains low frequency.
20. As a yadig user, I want copy execution to stop on auth, CSRF, captcha, rate-limit, or risk-control errors, so that unsafe retries do not continue.
21. As a yadig user, I want to configure an OpenAI-compatible LLM base URL, model, and API key, so that I can use my chosen provider.
22. As a yadig user, I want to test the LLM configuration before analyzing my library, so that invalid credentials or incompatible APIs are found early.
23. As a yadig user, I want the test result to distinguish authentication, network, API compatibility, and response-format failures, so that I know what to fix.
24. As a yadig user, I want the API key hidden by default and excluded from errors and logs, so that credentials are not exposed.
25. As a yadig user, I want LLM analysis to send only item metadata needed for classification, so that Bilibili cookies and account credentials never leave the app.
26. As a yadig user, I want large favorite folders analyzed in bounded chunks, so that requests remain within provider limits.
27. As a yadig user, I want partial chunk failures to remain visible, so that successful classifications are not discarded and failed items are not silently mislabeled.
28. As a yadig user, I want every classification to include a category, tags, reason, and confidence, so that I can evaluate the suggestion.
29. As a yadig user, I want the LLM to optionally suggest copy, move, delete, or no action, so that classification can assist organization.
30. As a yadig user, I want an LLM-suggested target folder represented as a suggestion, so that it can be reviewed before becoming a plan target.
31. As a yadig user, I want analysis results to identify whether they came from the LLM or local metadata rules, so that provenance is explicit.
32. As a yadig user, I want a failed explicit LLM analysis to report failure instead of silently replacing it with local fallback, so that I do not mistake heuristic results for model output.
33. As a yadig user, I want local metadata analysis to remain available as a separate explicit action, so that classification still works without an API.
34. As a yadig user, I want classification results persisted locally, so that I can review them after restarting the app.
35. As a yadig user, I want to filter favorite items by category, tag, confidence, suggested action, source folder, title, author, and Bilibili category, so that I can focus on a useful subset.
36. As a yadig user, I want to select all currently filtered items, so that I can prepare a batch quickly.
37. As a yadig user, I want to deselect individual items after selecting a category, so that I retain final control.
38. As a yadig user, I want suggested actions to prefill a draft action without executing it, so that LLM assistance reduces work without bypassing review.
39. As a yadig user, I want to override the suggested action or target folder before plan creation, so that incorrect suggestions are easy to correct.
40. As a yadig user, I want the plan preview to identify which items were selected from an LLM classification, so that the decision path remains auditable.
41. As a yadig user, I want all write operations to require a complete Bilibili session, so that missing CSRF or account identity blocks execution.
42. As a yadig user, I want stale local data warnings before folder or membership writes, so that I can resync before modifying the account.
43. As a yadig user, I want write errors sanitized across previews, current results, and history, so that cookies, CSRF tokens, API keys, callback URLs, and account identifiers are never displayed.
44. As a yadig user, I want follow and watch-later resources to remain readable in the workstation, so that this work does not regress existing sync behavior.
45. As a future yadig user, I want operation concepts that can later support follows and watch later, so that those features do not require replacing the entire plan/history model.

## Implementation Decisions

### Scope and Delivery Strategy

The feature is delivered as thin, independently verifiable vertical slices. Favorites remain the only remote-write resource in this PRD. Follow groups and watch-later mutation are deferred.

Existing move/delete behavior remains supported. New behavior extends rather than replaces the current local library, membership, analysis, and operation history concepts.

### Favorite Folder Service

A focused favorites service will own favorite folder and favorite resource write contracts. Tauri commands remain thin wrappers.

Supported folder actions:

- Create via `POST /x/v3/fav/folder/add`.
- Rename/update via `POST /x/v3/fav/folder/edit`.
- Delete via `POST /x/v3/fav/folder/del`.

Supported resource actions:

- Copy via `POST /x/v3/fav/resource/copy`.
- Existing move and batch delete actions remain unchanged.

These are Bilibili Web endpoints rather than stable public product contracts. Each new endpoint requires form-builder tests, sanitized error mapping, and a disposable-folder smoke test before broad use.

### Folder Metadata

Create supports title, introduction, and privacy. Cover customization is not required in this version.

Rename is title-focused but must preserve synchronized introduction, privacy, and cover metadata when sending the edit request. If required metadata is missing or stale, the UI asks the user to resync instead of guessing defaults.

The service must reject blank titles and titles that violate observed Bilibili constraints before sending a request. Remote validation errors remain authoritative and are surfaced safely.

### Folder Mutation Safety

Folder create and rename require an explicit confirmation in the preview but do not require typed confirmation.

Folder deletion is destructive and may delete a non-empty folder. It requires:

- Current folder title and remote id.
- Known item count and snapshot freshness.
- A preview of known item titles for non-empty folders.
- Exact confirmation text `DELETE <folder name>`.
- A complete Bilibili write session.

Folders identified as system, default, non-owned, or otherwise not safely mutable are blocked before execution.

Successful delete removes the local collection and its memberships. Library items remain if they have another membership or are needed by another local resource type; orphan cleanup must follow existing library ownership rules rather than deleting items unconditionally.

### Copy Semantics

Copy is a new operation-plan kind with the same remote resource identity requirements as move:

- Source folder id.
- Target folder id.
- Current account id.
- Resource id and resource type.

Copy preserves source membership and adds target membership after remote success. Copy to the same folder is skipped. If the target already contains the item, the result is treated as skipped/no-op when detectable from local or remote state.

Copy uses small source/target-homogeneous batches and pauses between successful batches. Blocking account/security errors stop remaining execution.

### Operation Plans and History

The operation model will support both resource operations and folder operations without erasing existing history.

New operation kinds include:

- Bilibili favorite copy.
- Bilibili favorite folder create.
- Bilibili favorite folder rename.
- Bilibili favorite folder delete.

Plans receive a stable client-generated identifier or execution-group identifier so a draft and its execution result can be associated. Existing numeric database ids remain valid.

Folder plans contain the affected folder identity and sanitized before/after metadata. Resource plans continue to contain per-item source, target, and remote resource identity.

LLM classification provenance may be recorded as a local reference or plan metadata, but prompts, API keys, cookies, and full raw provider responses are not copied into operation history.

### LLM Provider Contract

The first formal provider contract is OpenAI-compatible chat completions. Provider configuration contains:

- Provider label.
- Base URL.
- Model name.
- API key.

The intended initial profile uses `https://token-plan-cn.xiaomimimo.com/v1` and `mimo-v2.5-pro`. The API key is supplied through Settings and stored only in local Tauri Store.

A connection test sends a minimal, non-user-content request and validates:

- HTTP connectivity.
- Authentication.
- Chat-completions response shape.
- Ability to obtain parseable JSON content.

The provider client supports APIs that honor `response_format: json_object` and APIs that require prompt-only JSON output. Compatibility fallback changes request formatting, not result provenance.

### LLM Analysis Behavior

Explicit "Analyze with LLM" uses strict semantics:

- Missing or invalid configuration is an error.
- Provider failures are reported as LLM failures.
- Failed chunks are not silently replaced with local metadata classifications.
- Successful chunks remain available when another chunk fails.

Local metadata classification remains a separate explicit action.

Favorite items are analyzed in bounded chunks. Chunk size is fixed conservatively for the first version and can be adjusted later based on measured payload size.

Each result uses a validated structured contract:

- External item id.
- Category.
- Suggested tags.
- Reason.
- Confidence between 0 and 1.
- Suggested action: copy, move, delete, or none.
- Optional suggested target folder title.
- Provenance and analysis timestamp.

Unknown item ids, duplicate results, invalid confidence values, unsupported actions, and malformed target suggestions are rejected or normalized before persistence.

### Classification Persistence and Selection

LLM classifications deepen the existing analysis/tag model rather than creating an unrelated store. The database must retain category, action suggestion, target suggestion, provenance, model, and analysis timestamp in a queryable form.

Workstation exposes filters for:

- Source favorite folder.
- Category.
- Tag.
- Confidence threshold.
- Suggested action.
- Suggested target folder.
- Title, author, and Bilibili category.

Filtering never changes remote state. Users select the filtered subset, may remove individual items, choose or override an action and target folder, then create a normal operation-plan draft.

### Secret Handling

The LLM API key must not appear in:

- Source files or committed configuration.
- PRDs or issues.
- Console logs.
- Rust errors returned to the frontend.
- SQLite analysis records.
- Operation plans or history.

Sanitization covers bearer tokens and common API-key forms in addition to Bilibili cookies, CSRF values, callback URLs, and account ids.

The current Tauri Store approach is accepted for this scope. OS keychain storage remains future work.

### Frontend Experience

Workstation gains a focused favorite-folder manager and a classification review surface.

Folder manager actions:

- Create folder.
- Rename selected mutable folder.
- Delete selected folder with impact preview and typed confirmation.
- Refresh folders after mutation.

Classification review actions:

- Test LLM provider from Settings.
- Analyze the selected folder or selected items.
- Run explicit local metadata classification.
- Filter and select results.
- Prefill copy/move/delete drafts from suggestions.
- Override action and target before draft creation.

The UI must visibly distinguish suggestion, draft, executing, and executed states.

### Future Compatibility

Operation and analysis types should use resource/action vocabulary that can later represent follow groups and watch later, but no generic framework should be built beyond what the favorites slices need.

## Testing Decisions

Good tests verify observable contracts and safety behavior. They should not depend on private helper structure or real credentials.

Modules to test:

- Favorite folder request construction for create, edit, and delete.
- Folder mutation eligibility, metadata preservation, and typed-confirmation rules.
- Favorite copy request construction, batching, no-op detection, stop-on-blocked behavior, and local membership updates.
- Operation-plan serialization, stable draft/execution association, and backward compatibility with existing move/delete history.
- LLM provider connection-test response classification.
- OpenAI-compatible request construction with and without `response_format`.
- Structured classification parsing and validation.
- Strict LLM failure semantics and partial chunk results.
- Explicit metadata fallback provenance.
- API-key and Bilibili-secret redaction.
- Database persistence and filtering of category, confidence, suggestion, provenance, and timestamp.
- Typed frontend-to-Tauri contracts through TypeScript compilation and production build.

Prior art:

- Existing Bilibili client tests cover favorite form construction, complete-session requirements, normalization, batching, blocked-error handling, and redaction.
- Existing operation-plan tests cover move/delete plan creation and execution.
- Existing LLM tests cover OpenAI-compatible payload construction, JSON extraction, and metadata fallback.
- Existing frontend checks use TypeScript compilation and Vite production build; targeted frontend unit-test infrastructure may be introduced for pure filtering and classification-selection helpers.

Real remote writes are not unit tests. Disposable-folder smoke tests are required for:

- Create.
- Rename.
- Copy.
- Delete of an empty folder.
- Delete of a non-empty folder with known test items.

Smoke-test records must contain only folder aliases, test BV ids when safe, operation status, and sanitized response summaries.

## Out of Scope

- Managing Bilibili follow groups or changing follow relationships.
- Mutating watch-later state.
- Managing subscriptions, bangumi follows, channels, playlists, or UP dynamic subscriptions.
- Automatically executing any LLM recommendation.
- Allowing the LLM to create arbitrary remote folder names without user review.
- Favorite folder cover upload or image moderation workflows.
- Bulk delete of multiple folders in one user action.
- Cleaning invalid/deleted favorite resources through the clean endpoint.
- Browser automation or browser extension approaches.
- Circumventing captcha, risk control, account limits, or access restrictions.
- Replacing Tauri Store with OS keychain storage.
- General-purpose chat UI.

## Further Notes

This PRD builds on the completed Bilibili favorite remote operations MVP and the Bilibili session persistence commit `4440429`.

The endpoint contracts are based on the project's Bilibili batch-operations research and the community-maintained Bilibili API collection. They remain unofficial and must be treated as runtime-verified integrations.

The intended user workflow is:

1. Restore or verify Bilibili login.
2. Sync favorite folders and memberships.
3. Create or rename destination folders when needed.
4. Test the LLM provider.
5. Analyze a bounded set of favorite items.
6. Filter and manually select classification results.
7. Create a copy, move, or delete draft.
8. Review and confirm.
9. Execute slowly.
10. Review history and resync.
