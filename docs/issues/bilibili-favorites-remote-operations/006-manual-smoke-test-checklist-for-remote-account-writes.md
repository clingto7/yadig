# Issue 6: Manual smoke-test checklist for remote account writes

## Parent

PRD: `docs/prd/PRD-bilibili-favorites-remote-operations.md`

## Purpose

This checklist verifies Bilibili favorite remote writes with a deliberately small and disposable data set. Do not use existing important favorite folders or videos that matter to the account.

The smoke test covers only:

- Syncing favorite folders and memberships into yadig.
- Creating move and delete plans from the Workstation.
- Executing one small move plan and one small delete plan.
- Confirming the remote result on Bilibili Web.
- Recording sanitized app states and Bilibili response codes/messages.

## Test Data Rules

- [ ] Create two disposable Bilibili favorite folders on Bilibili Web:
  - `yadig-smoke-source-YYYYMMDD`
  - `yadig-smoke-target-YYYYMMDD`
- [ ] Add two or three non-important public test videos to the source folder.
- [ ] Do not include videos from existing personal folders unless losing that folder membership is acceptable.
- [ ] Keep the test batch small: one or two move items, then one delete item.
- [ ] Do not paste raw cookies, `SESSDATA`, `bili_jct`, `DedeUserID`, callback URLs, QR callback payloads, or account IDs into notes, screenshots, bug reports, or logs.

## Environment

- Date:
- Tester:
- yadig commit:
- OS:
- Bilibili account type:
- Disposable source folder:
- Disposable target folder:
- Test video BV IDs:

## Preflight

- [ ] Start yadig from a clean app session.
- [ ] Open Settings and verify Bilibili login state is authenticated.
- [ ] Use QR login or a full Cookie login that preserves write eligibility. SESSDATA-only login is read-only for these write tests.
- [ ] Confirm session status does not require re-login.
- [ ] Open Bilibili Web in a browser and confirm the disposable source and target folders exist.
- [ ] Confirm the source folder contains the selected test videos and the target folder is empty or contains only disposable test content.

Record any sanitized session/status note:

```text
status:
message:
```

## Sync Verification

- [ ] In Workstation, click `Sync Bilibili`.
- [ ] Confirm the local library shows favorite resources.
- [ ] Set Resource type to `Favorites`.
- [ ] Select the disposable source favorite folder.
- [ ] Confirm the test videos are visible with the source folder label.
- [ ] Confirm the disposable target folder is available in the move target selector.
- [ ] Do not proceed if the expected folders or test videos are missing. Re-sync or fix the remote test data first.

Record sync result:

```text
synced item count:
source folder visible: yes/no
target folder visible: yes/no
test videos visible: yes/no
sanitized message:
```

## Move Smoke Test

- [ ] Select one or two test videos from the disposable source folder.
- [ ] Choose the disposable target folder as the move target.
- [ ] Click `Move Draft`.
- [ ] Confirm Plan Preview shows:
  - action `Move`
  - source folder equals the disposable source folder
  - target folder equals the disposable target folder
  - item count equals the selected videos
  - executable items are `Pending`
- [ ] Confirm Favorite Operation History shows a recent `Move` plan with status `Draft` and the correct item count.
- [ ] Click `Execute Move` and confirm the browser prompt.
- [ ] Wait for execution to complete.
- [ ] Confirm successful items become `Success` in Plan Preview or Favorite Operation History.
- [ ] Open Bilibili Web and confirm the moved videos are now in the disposable target folder.
- [ ] Confirm the local Workstation library view reflects the target folder membership after execution.

Record move result:

```text
plan id/time:
selected BV IDs:
app states seen: preview / executing / success / failed / skipped / blocked
success count:
failed count:
skipped count:
blocked count:
sanitized Bilibili code/message:
sanitized app message:
remote web confirmation:
```

## Delete Smoke Test

- [ ] Pick one disposable test video in a disposable favorite folder. Prefer a video already moved into the disposable target folder.
- [ ] In Workstation, select the folder that currently contains that test video.
- [ ] Select exactly one test video.
- [ ] Click `Delete Draft`.
- [ ] Confirm Plan Preview shows:
  - action `Delete`
  - source folder equals the disposable folder currently containing the test video
  - no target folder
  - item count equals one
  - executable item is `Pending`
- [ ] Confirm Favorite Operation History shows a recent `Delete` plan with status `Draft` and one item.
- [ ] Click `Execute Delete`.
- [ ] Type `DELETE` only when the prompt references the disposable test favorite.
- [ ] Wait for execution to complete.
- [ ] Confirm the deleted membership becomes `Success` in Plan Preview or Favorite Operation History.
- [ ] Open Bilibili Web and confirm the video is no longer listed in that disposable favorite folder.
- [ ] Confirm the local Workstation library view no longer shows that folder membership for the deleted item.

Record delete result:

```text
plan id/time:
selected BV ID:
app states seen: preview / executing / success / failed / skipped / blocked
success count:
failed count:
skipped count:
blocked count:
sanitized Bilibili code/message:
sanitized app message:
remote web confirmation:
```

## Failure And Blocked-State Checks

Use one of these only if it can be done without risking important account data.

- [ ] Same-folder move: create a move draft where source and target are the same disposable folder. Expected item state is `Skipped`; execution should not be required for that item.
- [ ] Missing write eligibility: if testing with a read-only session, favorite write execution should be refused or blocked before remote mutation.
- [ ] Risk-control/auth failure: if Bilibili returns login, CSRF, captcha, rate-limit, or risk-control errors, execution should stop remaining pending items and mark them `Blocked`.
- [ ] Retryable failure: transient non-account failures should appear as `Failed` or retryable in history without hiding successful items.

Record sanitized failure details:

```text
scenario:
expected state:
actual state:
sanitized code:
sanitized message:
remaining pending/blocked items:
manual action needed:
```

## Expected App State Matrix

| State | Expected meaning | Tester check |
| --- | --- | --- |
| Preview | Draft plan exists; no remote write has run. | Source, target, title, action, and item count are correct. |
| Executing | User confirmed execution; yadig is applying a small batch. | UI is busy and does not allow duplicate execution. |
| Success | Bilibili accepted the operation and local library was updated. | Web state and Workstation membership match. |
| Failed | Item did not complete and may be retryable. | Error is sanitized and item remains visible after restart. |
| Skipped | No-op or stale item should not be retried as-is. | Reason is visible, such as same target folder. |
| Blocked | Account/session/risk-control state makes further calls unsafe. | Remaining items stop and show sanitized manual-action reason. |

## Restart Persistence Check

- [ ] Quit and restart yadig.
- [ ] Open Workstation.
- [ ] Confirm recent favorite operation plans remain visible in Favorite Operation History.
- [ ] Confirm failed, skipped, or blocked item details remain visible if any were produced.
- [ ] Confirm no raw cookie, CSRF token, callback URL, or account identifier appears in the UI.

## Sanitized Response Log

Paste only sanitized response data.

| Step | Remote operation | Sanitized code | Sanitized message | App state | Web confirmed |
| --- | --- | --- | --- | --- | --- |
| Move |  |  |  |  |  |
| Delete |  |  |  |  |  |
| Failure/blocked check |  |  |  |  |  |

## Cleanup

- [ ] Remove remaining disposable test videos from disposable folders if desired.
- [ ] Delete the disposable favorite folders on Bilibili Web if they are no longer needed.
- [ ] Keep only sanitized test notes in the issue or report.

## Acceptance criteria

- [x] The checklist instructs testers to use disposable favorite folders and non-important test videos.
- [x] The checklist covers login/session verification before write tests.
- [x] The checklist covers sync verification before plan creation.
- [x] The checklist covers moving one or two videos between disposable folders.
- [x] The checklist covers deleting one test favorite from a disposable folder.
- [x] The checklist covers confirming remote state on Bilibili Web after each operation.
- [x] The checklist records expected app states for preview, execution, success, failed, skipped, and blocked outcomes.
- [x] The checklist reminds testers not to paste cookies, CSRF tokens, callback URLs, or account IDs into logs.
- [x] The checklist includes a place to record sanitized Bilibili response codes/messages.

## Blocked by

- Issue 3: Execute favorite move plans
- Issue 4: Execute favorite delete plans
- Issue 5: Surface execution logs and retry states
