import type { LibraryCollection } from "@/lib/tauri";

export interface FavoriteFolderManagerSelection {
  folderRenameId: string;
  folderRenameTitle: string;
  folderDeleteId: string;
}

export function syncFavoriteFolderManagerSelection(
  folders: LibraryCollection[],
  selectedFolderId: string,
  current: FavoriteFolderManagerSelection
): FavoriteFolderManagerSelection {
  const folderById = new Map(folders.map((folder) => [folder.externalId, folder]));
  const selectedFolder = selectedFolderId === "all" ? null : folderById.get(selectedFolderId) ?? null;

  if (selectedFolder) {
    const preserveRenameTitle = current.folderRenameId === selectedFolder.externalId;
    return {
      folderRenameId: selectedFolder.externalId,
      folderRenameTitle: preserveRenameTitle ? current.folderRenameTitle : selectedFolder.title,
      folderDeleteId: selectedFolder.externalId,
    };
  }

  const renameFolder = folderById.get(current.folderRenameId);
  const deleteFolder = folderById.get(current.folderDeleteId);

  return {
    folderRenameId: renameFolder ? current.folderRenameId : "",
    folderRenameTitle: renameFolder ? current.folderRenameTitle : "",
    folderDeleteId: deleteFolder ? current.folderDeleteId : "",
  };
}
