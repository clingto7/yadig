import { qrLoginUiState } from "@/lib/bili-login-ui";

const expired = qrLoginUiState(86038);
if (!expired.expired || expired.message !== "QR code expired.") {
  throw new Error("Expired QR codes must surface a refreshable expired state.");
}

const scanned = qrLoginUiState(86090);
if (scanned.expired || !scanned.waiting || !scanned.message.includes("confirm")) {
  throw new Error("Scanned QR codes should keep polling while waiting for confirmation.");
}

const success = qrLoginUiState(0);
if (success.expired || success.waiting || !success.message.includes("successful")) {
  throw new Error("Successful QR codes should stop waiting without showing expiration.");
}
