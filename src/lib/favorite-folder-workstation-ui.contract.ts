import type { LibraryCollection } from "@/lib/tauri";
import { syncFavoriteFolderManagerSelection } from "@/lib/favorite-folder-workstation-ui";

const folders: LibraryCollection[] = [
  {
    source: "bilibili",
    externalId: "100",
    collectionType: "bili_favorite_folder",
    title: "Archive",
    rawMetadata: {},
  },
  {
    source: "bilibili",
    externalId: "200",
    collectionType: "bili_favorite_folder",
    title: "Music",
    rawMetadata: {},
  },
];

const syncedFromReviewFolder = syncFavoriteFolderManagerSelection(folders, "200", {
  folderRenameId: "",
  folderRenameTitle: "",
  folderDeleteId: "",
});

if (
  syncedFromReviewFolder.folderRenameId !== "200" ||
  syncedFromReviewFolder.folderRenameTitle !== "Music" ||
  syncedFromReviewFolder.folderDeleteId !== "200"
) {
  throw new Error("Selecting a specific review folder should prefill rename and delete folder controls.");
}

const preservedRenameDraft = syncFavoriteFolderManagerSelection(folders, "200", {
  folderRenameId: "200",
  folderRenameTitle: "Music Archive",
  folderDeleteId: "200",
});

if (preservedRenameDraft.folderRenameTitle !== "Music Archive") {
  throw new Error("Editing a rename title should not be reset while the same folder stays selected.");
}

const preservedBlankRenameDraft = syncFavoriteFolderManagerSelection(folders, "200", {
  folderRenameId: "200",
  folderRenameTitle: "",
  folderDeleteId: "200",
});

if (preservedBlankRenameDraft.folderRenameTitle !== "") {
  throw new Error("Blank rename drafts should remain editable for validation instead of being reset.");
}

const clearedStaleFolder = syncFavoriteFolderManagerSelection(folders, "all", {
  folderRenameId: "missing",
  folderRenameTitle: "Missing",
  folderDeleteId: "missing",
});

if (
  clearedStaleFolder.folderRenameId !== "" ||
  clearedStaleFolder.folderRenameTitle !== "" ||
  clearedStaleFolder.folderDeleteId !== ""
) {
  throw new Error("Missing folder ids should be cleared from folder operation controls.");
}
