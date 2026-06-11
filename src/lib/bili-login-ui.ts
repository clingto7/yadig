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
