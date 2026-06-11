import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

export interface DownloadSavedNotification {
  title: string;
  body: string;
}

export interface DownloadNotificationDeps {
  isPermissionGranted: () => Promise<boolean>;
  requestPermission: () => Promise<NotificationPermission>;
  sendNotification: (options: DownloadSavedNotification) => void;
}

const defaultDownloadNotificationDeps: DownloadNotificationDeps = {
  isPermissionGranted,
  requestPermission,
  sendNotification,
};

export function buildDownloadSavedNotification(path: string): DownloadSavedNotification {
  return {
    title: "Download saved",
    body: `Saved to ${path}`,
  };
}

export async function notifyDownloadSaved(
  path: string,
  deps: DownloadNotificationDeps = defaultDownloadNotificationDeps
): Promise<boolean> {
  try {
    let granted = await deps.isPermissionGranted();
    if (!granted) {
      granted = (await deps.requestPermission()) === "granted";
    }
    if (!granted) return false;

    deps.sendNotification(buildDownloadSavedNotification(path));
    return true;
  } catch {
    return false;
  }
}
