import type { FavoriteFolderDeletePlanRequest, OperationPlan } from "@/lib/tauri";
import { tauri } from "@/lib/tauri";
import { deleteBiliFavoriteFolderFromPlan } from "@/lib/db";

const request: FavoriteFolderDeletePlanRequest = {
  folder: {
    source: "bilibili",
    externalId: "300",
    collectionType: "bili_favorite_folder",
    title: "Old folder",
    rawMetadata: {
      id: 300,
      title: "Old folder",
      media_count: 2,
      snapshotLastSyncedAt: "2026-06-11T10:00:00Z",
    },
  },
  knownItemCount: 2,
  knownItemTitles: ["Video A", "Video B"],
  snapshotLastSyncedAt: "2026-06-11T10:00:00Z",
};

const plan: OperationPlan = {
  kind: "bili_favorite_folder_delete",
  items: [
    {
      externalId: "300",
      title: request.folder.title,
      action: "delete_folder",
      target: null,
      status: "success",
      error: null,
      sourceCollectionExternalId: "300",
      sourceCollectionTitle: request.folder.title,
      targetCollectionExternalId: null,
      targetCollectionTitle: null,
      resourceId: null,
      resourceType: null,
      metadata: {
        knownItemCount: request.knownItemCount,
        knownItemTitles: request.knownItemTitles,
        snapshotLastSyncedAt: request.snapshotLastSyncedAt,
      },
    },
  ],
};

export const favoriteFolderDeleteOperationContract = {
  request,
  createDraft: () => tauri.createBiliFavoriteFolderDeletePlan(request),
  execute: () => tauri.executeBiliFavoriteFolderDeletePlan({
    plan,
    confirmationText: "DELETE Old folder",
  }),
  deleteLocalFolder: () => deleteBiliFavoriteFolderFromPlan(plan),
};
