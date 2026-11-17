/**
 * B01：共用 TypeScript → Rust Contract。
 * 這裡不包含 Android、iOS 專用 SDK；後續 Native Adapter 接在 Rust 邊界。
 */
export interface BridgeHealth {
  contractVersion: 1;
  bridge: "ready";
  platform: string;
  appVersion: string;
  networkRequired: false;
  cloudAiEnabled: false;
  nativeAi: "not_installed";
}

export type InvokeAdapter = (command: string) => Promise<unknown>;

export function parseBridgeHealth(value: unknown): BridgeHealth {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("Rust Bridge 回傳格式錯誤");
  }

  const report = value as Record<string, unknown>;
  if (
    report.contractVersion !== 1 ||
    report.bridge !== "ready" ||
    typeof report.platform !== "string" ||
    report.platform.length === 0 ||
    typeof report.appVersion !== "string" ||
    report.appVersion.length === 0 ||
    report.networkRequired !== false ||
    report.cloudAiEnabled !== false ||
    report.nativeAi !== "not_installed"
  ) {
    throw new Error("Rust Bridge Contract 不符合 B01／No Cloud AI 要求");
  }

  return report as unknown as BridgeHealth;
}

export async function checkBridge(invoke: InvokeAdapter): Promise<BridgeHealth> {
  const result = await invoke("bridge_health");
  return parseBridgeHealth(result);
}
