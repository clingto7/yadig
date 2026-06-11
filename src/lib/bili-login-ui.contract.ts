import { biliAccountTierLabel, formatBiliLoginError, qrLoginUiState } from "@/lib/bili-login-ui";

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

const missingSessdata = formatBiliLoginError("cookie", "Cookie login requires SESSDATA.");
if (
  missingSessdata !==
  "Cookie login needs SESSDATA. Paste a full Bilibili Cookie header or a SESSDATA value."
) {
  throw new Error("Cookie login should explain how to provide SESSDATA.");
}

const qrNetwork = formatBiliLoginError(
  "qr-poll",
  "Network error: QR poll failed: error sending request"
);
if (
  qrNetwork !== "Could not reach Bilibili. Check your network or proxy and try again."
) {
  throw new Error("QR network failures should be mapped to an actionable message.");
}

const passwordCaptcha = formatBiliLoginError(
  "password",
  "Login failed (-105): CAPTCHA required. If CAPTCHA is required, use Cookie Login instead."
);
if (
  passwordCaptcha !==
  "Password login may require CAPTCHA or risk verification. Use QR login or Cookie login instead."
) {
  throw new Error("Password CAPTCHA failures should suggest QR or Cookie login.");
}

const sanitized = formatBiliLoginError(
  "cookie",
  "risk control blocked SESSDATA=secret bili_jct=csrf callback=https://example.invalid/callback?code=1 DedeUserID=12345678"
);
if (
  sanitized.includes("secret") ||
  sanitized.includes("csrf") ||
  sanitized.includes("callback") ||
  sanitized.includes("12345678")
) {
  throw new Error("Login errors must not leak cookie, token, callback, or user-id fragments.");
}
