import type { FavoriteFolderRenamePlanRequest, OperationPlan } from "@/lib/tauri";
import { tauri } from "@/lib/tauri";
import { updateBiliFavoriteFolderFromRenamePlan } from "@/lib/db";

const request: FavoriteFolderRenamePlanRequest = {
  folder: {
    source: "bilibili",
    externalId: "300",
    collectionType: "bili_favorite_folder",
    title: "Old folder",
    rawMetadata: {
      id: 300,
      title: "Old folder",
      intro: "Keep this intro",
      privacy: 1,
      cover: "https://i0.hdslb.com/cover.jpg",
    },
  },
  newTitle: "New folder",
};

const plan: OperationPlan = {
  kind: "bili_favorite_folder_rename",
  items: [
    {
      externalId: "300",
      title: "New folder",
      action: "rename_folder",
      target: request.newTitle,
      status: "success",
      error: null,
      sourceCollectionExternalId: "300",
      sourceCollectionTitle: request.folder.title,
      targetCollectionExternalId: "300",
      targetCollectionTitle: request.newTitle,
      resourceId: null,
      resourceType: null,
      metadata: request.folder.rawMetadata,
    },
  ],
};

export const favoriteFolderRenameOperationContract = {
  request,
  createDraft: () => tauri.createBiliFavoriteFolderRenamePlan(request),
  execute: () => tauri.executeBiliFavoriteFolderRenamePlan({ plan, confirmed: true }),
  updateLocalFolder: () => updateBiliFavoriteFolderFromRenamePlan(plan),
};
