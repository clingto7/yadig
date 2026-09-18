import type {
  OperationPlanItem,
  OperationPlanItemStatus,
} from "@/lib/tauri";

export type OperationPlanHistoryStatus =
  | "draft"
  | "running"
  | "completed"
  | "partial"
  | "failed"
  | "blocked";

export type OperationIssueKind = "none" | "retryable" | "blocked";

const ITEM_STATUSES: OperationPlanItemStatus[] = [
  "pending",
  "running",
  "success",
  "skipped",
  "failed",
  "blocked",
];

const STATUS_LABELS: Record<OperationPlanItemStatus, string> = {
  pending: "Pending",
  running: "Running",
  success: "Success",
  skipped: "Skipped",
  failed: "Failed",
  blocked: "Blocked",
};

const PLAN_STATUS_LABELS: Record<OperationPlanHistoryStatus, string> = {
  draft: "Draft",
  running: "Running",
  completed: "Completed",
  partial: "Partial",
  failed: "Failed",
  blocked: "Blocked",
};

export function operationPlanItemStatusLabel(status: OperationPlanItemStatus): string {
  return STATUS_LABELS[status];
}

export function normalizeOperationPlanItemStatus(status: string): OperationPlanItemStatus {
  return ITEM_STATUSES.includes(status as OperationPlanItemStatus)
    ? status as OperationPlanItemStatus
    : "failed";
}

export function operationPlanHistoryStatusLabel(status: OperationPlanHistoryStatus): string {
  return PLAN_STATUS_LABELS[status];
}

export function operationIssueLabel(kind: OperationIssueKind): string {
  switch (kind) {
    case "blocked":
      return "Manual action";
    case "retryable":
      return "Retryable";
    case "none":
      return "No issue";
  }
}

export function sanitizeOperationError(error: string | null | undefined): string | null {
  if (!error) return null;

  const sanitized = error
    .replace(/SESSDATA\s*=\s*[^;\s,)]+/gi, "SESSDATA=[redacted]")
    .replace(/bili_jct\s*=\s*[^;\s,)]+/gi, "bili_jct=[redacted]")
    .replace(/DedeUserID\s*=\s*\d+/gi, "DedeUserID=[redacted]")
    .replace(/https?:\/\/[^\s,)]+callback[^\s,)]+/gi, "[redacted callback URL]")
    .replace(/\bmid\s*[:=]\s*\d{5,}\b/gi, "mid=[redacted]")
    .replace(/\buid\s*[:=]\s*\d{5,}\b/gi, "uid=[redacted]");

  return sanitized.trim() || null;
}

export function classifyOperationIssue(item: Pick<OperationPlanItem, "status" | "error">): OperationIssueKind {
  if (item.status === "blocked") return "blocked";
  if (item.status !== "failed") return "none";

  const message = item.error?.toLowerCase() ?? "";
  if (
    message.includes("risk")
    || message.includes("captcha")
    || message.includes("csrf")
    || message.includes("login")
    || message.includes("session")
    || message.includes("blocked")
    || message.includes("security")
    || message.includes("账号")
    || message.includes("登录")
    || message.includes("风控")
  ) {
    return "blocked";
  }

  return "retryable";
}

export function deriveOperationPlanStatus<T extends Pick<OperationPlanItem, "status">>(
  items: T[]
): OperationPlanHistoryStatus {
  if (items.length === 0) return "draft";

  const statuses = new Set(items.map((item) => item.status));
  if (statuses.has("running")) return "running";
  if (statuses.has("blocked")) return "blocked";

  const successLikeCount = items.filter((item) => item.status === "success" || item.status === "skipped").length;
  const failedCount = items.filter((item) => item.status === "failed").length;
  const pendingCount = items.filter((item) => item.status === "pending").length;

  if (successLikeCount === items.length) return "completed";
  if (failedCount === items.length) return "failed";
  if (pendingCount === items.length) return "draft";
  if (failedCount > 0 || successLikeCount > 0) return "partial";
  return "draft";
}
