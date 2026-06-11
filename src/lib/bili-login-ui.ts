export type BiliQrPollCode = 0 | 86090 | 86038 | 86101 | number;

export interface BiliQrUiState {
  message: string;
  expired: boolean;
  waiting: boolean;
}

export function qrLoginUiState(code: BiliQrPollCode): BiliQrUiState {
  if (code === 0) {
    return { message: "Login successful!", expired: false, waiting: false };
  }
  if (code === 86090) {
    return { message: "Scanned - confirm on your phone", expired: false, waiting: true };
  }
  if (code === 86038) {
    return { message: "QR code expired.", expired: true, waiting: false };
  }
  return { message: "Waiting for scan...", expired: false, waiting: true };
}

export function biliAccountTierLabel(isPremium: boolean): "Premium" | "Standard" {
  return isPremium ? "Premium" : "Standard";
}

export type BiliLoginErrorContext = "qr-start" | "qr-poll" | "cookie" | "password" | "logout";

export function formatBiliLoginError(context: BiliLoginErrorContext, error: unknown): string {
  const raw = stringifyLoginError(error);
  const normalized = raw.toLowerCase();

  if (context === "cookie") {
    if (normalized.includes("requires sessdata") || normalized.includes("missing sessdata")) {
      return "Cookie login needs SESSDATA. Paste a full Bilibili Cookie header or a SESSDATA value.";
    }
    if (normalized.includes("enter a bilibili cookie") || normalized.includes("sessdata value")) {
      return "Cookie login needs a Cookie header or SESSDATA value.";
    }
    return "Cookie login failed. Check that the Cookie header or SESSDATA value is current and try again.";
  }

  if (context === "password") {
    if (normalized.includes("username and password cannot be empty")) {
      return "Enter your Bilibili username and password.";
    }
    if (
      normalized.includes("captcha") ||
      normalized.includes("risk") ||
      normalized.includes("verification")
    ) {
      return "Password login may require CAPTCHA or risk verification. Use QR login or Cookie login instead.";
    }
    if (isNetworkLoginError(normalized)) {
      return "Could not reach Bilibili. Check your network or proxy and try again.";
    }
    return "Password login failed. Check your credentials, or use QR login or Cookie login instead.";
  }

  if (context === "qr-start" || context === "qr-poll") {
    if (isNetworkLoginError(normalized)) {
      return "Could not reach Bilibili. Check your network or proxy and try again.";
    }
    return context === "qr-start"
      ? "Could not start QR login. Try again in a moment."
      : "Could not check QR login status. Refresh the QR code and try again.";
  }

  return "Could not log out. Try again.";
}

function stringifyLoginError(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string") {
    return error;
  }
  if (error == null) {
    return "";
  }
  return String(error);
}

function isNetworkLoginError(normalizedError: string): boolean {
  return (
    normalizedError.includes("network") ||
    normalizedError.includes("failed to fetch") ||
    normalizedError.includes("failed: error sending request") ||
    normalizedError.includes("timed out") ||
    normalizedError.includes("timeout") ||
    normalizedError.includes("dns") ||
    normalizedError.includes("connection")
  );
}
