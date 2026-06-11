import {
  buildDownloadSavedNotification,
  notifyDownloadSaved,
  type DownloadNotificationDeps,
} from "@/lib/download-notification";

const message = buildDownloadSavedNotification("/Users/mizu/Downloads/yadig/song.m4a");
if (message.title !== "Download saved" || !message.body.includes("song.m4a")) {
  throw new Error("Download notifications should include a stable title and saved path.");
}

const sent: unknown[] = [];
const grantedDeps: DownloadNotificationDeps = {
  isPermissionGranted: async () => true,
  requestPermission: async () => "denied",
  sendNotification: (options) => sent.push(options),
};

const sentWhenGranted = await notifyDownloadSaved("/tmp/song.m4a", grantedDeps);
if (!sentWhenGranted || sent.length !== 1) {
  throw new Error("Download notification should be sent when permission is already granted.");
}

const deniedDeps: DownloadNotificationDeps = {
  isPermissionGranted: async () => false,
  requestPermission: async () => "denied",
  sendNotification: (options) => sent.push(options),
};

const sentWhenDenied = await notifyDownloadSaved("/tmp/denied.m4a", deniedDeps);
if (sentWhenDenied || sent.length !== 1) {
  throw new Error("Download notification should not be sent when permission is denied.");
}
