import type { OperationPlan } from "@/lib/tauri";
import { tauri } from "@/lib/tauri";
import { updateBiliFavoriteCopyMemberships } from "@/lib/db";

const copyPlan: OperationPlan = {
  kind: "bili_batch_copy",
  items: [
    {
      externalId: "BVcopy",
      title: "Copy candidate",
      action: "copy",
      target: "200",
      status: "success",
      error: null,
      sourceCollectionExternalId: "100",
      sourceCollectionTitle: "Inbox",
      targetCollectionExternalId: "200",
      targetCollectionTitle: "Samples",
      resourceId: "987654321",
      resourceType: "2",
    },
  ],
};

export const favoriteCopyOperationContract = {
  planKind: copyPlan.kind,
  execute: () => tauri.executeBiliFavoriteCopyPlan({ plan: copyPlan, confirmed: true }),
  updateLocalMemberships: () => updateBiliFavoriteCopyMemberships(copyPlan),
};
