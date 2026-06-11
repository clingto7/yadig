import { biliAccountTierLabel, qrLoginUiState } from "@/lib/bili-login-ui";

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

if (biliAccountTierLabel(true) !== "Premium") {
  throw new Error("Premium accounts must show a Premium tier label.");
}

if (biliAccountTierLabel(false) !== "Standard") {
  throw new Error("Standard accounts must show a Standard tier label.");
}
