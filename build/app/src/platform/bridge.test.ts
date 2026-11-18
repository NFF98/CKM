import { describe, expect, it, vi } from "vitest";
import { checkBridge, parseBridgeHealth } from "./bridge";

const androidHealth = {
  contractVersion: 1,
  bridge: "ready",
  platform: "android",
  appVersion: "0.1.0",
  networkRequired: false,
  cloudAiEnabled: false,
  nativeAi: "not_installed",
} as const;

describe("S1-AC01：TS↔Rust Bridge 契約", () => {
  it("使用真正命名的 Rust Command 並驗證回應", async () => {
    const invoke = vi.fn().mockResolvedValue(androidHealth);
    await expect(checkBridge(invoke)).resolves.toEqual(androidHealth);
    expect(invoke).toHaveBeenCalledOnce();
    expect(invoke).toHaveBeenCalledWith("bridge_health");
  });

  it("拒絕傳入外部雲端 AI 功能開關", () => {
    expect(() => parseBridgeHealth({ ...androidHealth, cloudAiEnabled: true }))
      .toThrow(/No Cloud AI/);
  });

  it("拒絕假回應或錯誤 contract version", () => {
    expect(() => parseBridgeHealth(null)).toThrow();
    expect(() => parseBridgeHealth({ ...androidHealth, contractVersion: 2 }))
      .toThrow(/Contract/);
  });

  it("Rust invoke 失敗不能顯示成功", async () => {
    const invoke = vi.fn().mockRejectedValue(new Error("bridge unavailable"));
    await expect(checkBridge(invoke)).rejects.toThrow("bridge unavailable");
  });

  it("已下載且已驗證的 Gemma 必須顯示真實模型狀態", () => {
    const result = parseBridgeHealth({ ...androidHealth, nativeAi: "model_verified" });
    expect(result.nativeAi).toBe("model_verified");
  });

  it("尚未下載、等待 Wi-Fi、下載中或暫停都不能假裝已安裝", () => {
    for (const nativeAi of ["not_installed", "waiting_wifi", "downloading", "paused",
      "download_error", "unavailable", "status_error"]) {
      expect(parseBridgeHealth({ ...androidHealth, nativeAi }).nativeAi).toBe(nativeAi);
    }
    expect(() => parseBridgeHealth({ ...androidHealth, nativeAi: "cloud_ready" }))
      .toThrow("Contract");
  });

  it("共用 contract 接受 iOS 平台，但不宣稱 iOS 已測試", () => {
    expect(parseBridgeHealth({ ...androidHealth, platform: "ios" }).platform)
      .toBe("ios");
  });
});
