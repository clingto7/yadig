import type { FavoriteFolderCreatePlanRequest, OperationPlan } from "@/lib/tauri";
import { tauri } from "@/lib/tauri";
import { upsertBiliFavoriteFolderFromCreatePlan } from "@/lib/db";

const request: FavoriteFolderCreatePlanRequest = {
  title: "Disposable",
  intro: "Temporary test folder",
  privacy: "private",
};

const plan: OperationPlan = {
  kind: "bili_favorite_folder_create",
  items: [
    {
      externalId: "300",
      title: request.title,
      action: "create_folder",
      target: "1",
      status: "success",
      error: null,
      sourceCollectionExternalId: null,
      sourceCollectionTitle: null,
      targetCollectionExternalId: "300",
      targetCollectionTitle: request.intro,
      resourceId: null,
      resourceType: null,
    },
  ],
};

export const favoriteFolderCreateOperationContract = {
  request,
  createDraft: () => tauri.createBiliFavoriteFolderCreatePlan(request),
  execute: () => tauri.executeBiliFavoriteFolderCreatePlan({ plan, confirmed: true }),
  updateLocalFolder: () => upsertBiliFavoriteFolderFromCreatePlan(plan),
};
