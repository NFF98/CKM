import { describe, expect, it, vi } from "vitest";
import {
  importModel, loadModelSuggestion, modelStatus, parseModelStatus, proposeModel,
  prepareModel, pauseModelDownload
} from "./local-model";

const id = "2bc595b3-a0e4-4254-bc8f-200b1b0f19a2";
const ready = {
  installed: true,
  modelId: "gemma-4-E2B-it",
  runtime: "litert-lm-android-0.16.1",
  bytes: 2_588_147_712,
  integrityVerified: true,
  downloadState: "ready",
  downloadedBytes: 2_588_147_712,
  totalBytes: 2_588_147_712,
  downloadError: null,
};
const proposal = {
  assetId: id,
  status: "suggested_private",
  evidenceQuote: "パクる",
  contentHint: "日語詞彙",
  uncertainty: null,
  modelId: "gemma-4-E2B-it",
  runtime: "litert-lm-android-0.16.1",
  backend: "cpu",
  elapsedMs: 18700,
  updatedAtMs: 1794890000000,
};

describe("B03 本地 Gemma 4 E2B Model Contract", () => {
  it("只能讀真實 Rust 模型狀態，未安裝不裝作已安裝", async () => {
    const invoke = vi.fn().mockResolvedValue({
      ...ready, installed: false, bytes: 0, integrityVerified: false,
      downloadState: "idle", downloadedBytes: 0
    });
    expect((await modelStatus(invoke)).installed).toBe(false);
    expect(invoke).toHaveBeenCalledWith("local_model_status");
  });

  it("首次開啟只從 Rust 啟動官方模型下載，不送使用者內容", async () => {
    const downloading = { ...ready, installed: false, bytes: 0, integrityVerified: false,
      downloadState: "downloading", downloadedBytes: 1_048_576 };
    const invoke = vi.fn().mockResolvedValue(downloading);
    await expect(prepareModel(invoke)).resolves.toEqual(downloading);
    expect(invoke).toHaveBeenCalledWith("prepare_local_model", { allowMetered: false });
    expect(invoke).toHaveBeenCalledTimes(1);
  });

  it("行動數據只能由本人明確操作，不能預設消耗", async () => {
    const waiting = { ...ready, installed: false, bytes: 0, integrityVerified: false,
      downloadState: "waiting_wifi", downloadedBytes: 0 };
    const invoke = vi.fn().mockResolvedValue(waiting);
    await prepareModel(invoke, true);
    expect(invoke).toHaveBeenCalledWith("prepare_local_model", { allowMetered: true });
  });

  it("暫停後保留 partial 進度；不會誤報下載完成", async () => {
    const paused = { ...ready, installed: false, bytes: 0, integrityVerified: false,
      downloadState: "paused", downloadedBytes: 1_500_000_000 };
    const invoke = vi.fn().mockResolvedValue(paused);
    const status = await pauseModelDownload(invoke);
    expect(status.downloadState).toBe("paused");
    expect(status.downloadedBytes).toBe(1_500_000_000);
    expect(invoke).toHaveBeenCalledWith("pause_local_model_download");
  });

  it("SHA 未驗證、假完成、超額進度一律拒絕", () => {
    for(const invalid of [
      { ...ready, downloadedBytes: 2_588_147_713 },
      { ...ready, downloadState: "downloading" },
      { ...ready, installed: false, bytes: 0, integrityVerified: false,
        downloadState: "ready" },
      { ...ready, downloadError: 123 },
    ]) expect(() => parseModelStatus(invalid)).toThrow("Contract");
  });

  it("使用 Android 原生 SAF 匯入，不向 WebView 洩漏本機模型路徑", async () => {
    const invoke = vi.fn().mockResolvedValue(ready);
    await expect(importModel(invoke)).resolves.toEqual(ready);
    expect(invoke).toHaveBeenCalledWith("import_local_model");
  });

  it("只傳已保存的私人 UUID，模型讀入由 Rust 控制", async () => {
    const invoke = vi.fn().mockResolvedValue(proposal);
    await expect(proposeModel(invoke, id)).resolves.toEqual(proposal);
    expect(invoke).toHaveBeenCalledWith("local_model_propose", { id });
  });

  it("不得把模型建議偽裝已確認 Goal／分享", async () => {
    const invoke = vi.fn().mockResolvedValue({ ...proposal, status: "CONFIRMED" });
    await expect(proposeModel(invoke, id)).rejects.toThrow("Contract");
    expect(invoke).toHaveBeenCalledTimes(1);
  });

  it("未安裝或 Cloud runtime 不能被當作本地完成", () => {
    expect(() => parseModelStatus({ ...ready, installed: true, integrityVerified: false })).toThrow();
    expect(() => parseModelStatus({ ...ready, runtime: "cloud-ai" })).toThrow();
  });

  it("已有的私有候選可從 SQLite 重讀；無候選為 null", async () => {
    const invoke = vi.fn().mockResolvedValue(proposal);
    await expect(loadModelSuggestion(invoke, id)).resolves.toEqual(proposal);
    expect(invoke).toHaveBeenCalledWith("load_model_suggestion", { id });
    invoke.mockResolvedValueOnce(null);
    await expect(loadModelSuggestion(invoke, id)).resolves.toBeNull();
  });

  it("錯誤 UUID 不會進入 Rust 命令", async () => {
    const invoke = vi.fn();
    await expect(proposeModel(invoke, "../../private")).rejects.toThrow("INVALID_ASSET_ID");
    expect(invoke).not.toHaveBeenCalled();
  });

  it("原生模型拒絕後不呼叫任何 Cloud fallback", async () => {
    const invoke = vi.fn().mockRejectedValue("MODEL_NOT_INSTALLED");
    await expect(proposeModel(invoke, id)).rejects.toBe("MODEL_NOT_INSTALLED");
    expect(invoke).toHaveBeenCalledTimes(1);
  });
});
