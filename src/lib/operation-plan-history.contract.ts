import type { OperationPlanItem } from "@/lib/tauri";
import {
  classifyOperationIssue,
  deriveOperationPlanStatus,
  operationIssueLabel,
  operationPlanHistoryStatusLabel,
  operationPlanItemStatusLabel,
  sanitizeOperationError,
} from "@/lib/operation-plan-history";

const blockedItem: OperationPlanItem = {
  externalId: "BV1contract",
  title: "Contract video",
  action: "move_favorite",
  target: "Archive",
  status: "blocked",
  error: "risk control blocked SESSDATA=secret bili_jct=csrf callback=https://example.invalid/callback?code=1 DedeUserID=12345678",
  sourceCollectionExternalId: "source-folder",
  sourceCollectionTitle: "Source",
  targetCollectionExternalId: "target-folder",
  targetCollectionTitle: "Target",
  resourceId: "resource-id",
  resourceType: "2",
  metadata: {},
};

export const operationPlanHistoryContract = {
  blockedIssue: operationIssueLabel(classifyOperationIssue(blockedItem)),
  completedStatus: operationPlanHistoryStatusLabel(
    deriveOperationPlanStatus([{ ...blockedItem, status: "success", error: null }])
  ),
  itemStatus: operationPlanItemStatusLabel(blockedItem.status),
  sanitizedError: sanitizeOperationError(blockedItem.error),
};
