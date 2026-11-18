import { describe, it, expect, vi } from "vitest";
import { correctOcr, loadOcr, parseOcrRecord, recognizeOcr } from "./local-ocr";

const id = "2bc595b3-a0e4-4254-bc8f-200b1b0f19a2";
const data = {
  assetId: id,
  status: "recognized",
  rawText: "パクる",
  correctedText: null,
  engine: "mlkit_japanese_bundled",
  engineVersion: "16.0.1",
  segments: [{ text: "パクる", bounds: [0, 0, 120, 48] }],
  elapsedMs: 75,
  updatedAtMs: 1794890000000,
};

describe("B03 日文 OCR 及更正 Contract（非真機模型測試）", () => {
  it("Android OCR 只向 Rust 傳圖片 UUID，不向網路 API 傳原圖", async () => {
    const invoke = vi.fn().mockResolvedValue(data);
    await expect(recognizeOcr(invoke, id)).resolves.toEqual(data);
    expect(invoke).toHaveBeenCalledWith("local_ocr_recognize", { id });
  });

  it("原圖辨識結果與人工更正分欄，重開 App 只向 SQLite 讀取", async () => {
    const corrected = { ...data, correctedText: "パクる（模仿）" };
    const invoke = vi.fn().mockResolvedValue(corrected);
    await expect(correctOcr(invoke, id, "パクる（模仿）")).resolves.toEqual(corrected);
    expect(invoke).toHaveBeenCalledWith("correct_ocr", { id, correctedText: "パクる（模仿）" });
    await expect(loadOcr(invoke, id)).resolves.toEqual(corrected);
    expect(invoke).toHaveBeenCalledWith("load_ocr", { id });
  });

  it("OCR UNKNOWN 必須真顯示未知，不能憑內容猜 Goal", () => {
    expect(parseOcrRecord({
      ...data, status: "unknown", rawText: "", correctedText: null, segments: []
    }).status).toBe("unknown");
  });

  it("拒絕無效 ID、空白更正及過大輸入", async () => {
    const invoke = vi.fn();
    await expect(recognizeOcr(invoke, "../../secret")).rejects.toThrow("INVALID_ASSET_ID");
    await expect(correctOcr(invoke, id, "   ")).rejects.toThrow("EMPTY_TEXT");
    await expect(correctOcr(invoke, id, "日".repeat(23000))).rejects.toThrow("TEXT_TOO_LARGE");
    expect(invoke).not.toHaveBeenCalled();
  });

  it("拒絕偽造的 OCR Contract 及缺少狀態", () => {
    expect(() => parseOcrRecord({ ...data, status: "CONFIRMED" })).toThrow();
    expect(() => parseOcrRecord({ ...data, assetId: "../../db" })).toThrow();
    expect(() => parseOcrRecord({ ...data, segments: [{text: "a", bounds: ["bad"]}] })).toThrow();
  });

  it("本地 OCR 拒絕時不宣稱成功或切換 Cloud AI", async () => {
    const invoke = vi.fn().mockRejectedValue("OCR_RECOGNITION_FAILED");
    await expect(recognizeOcr(invoke, id)).rejects.toBe("OCR_RECOGNITION_FAILED");
    expect(invoke).toHaveBeenCalledTimes(1);
  });
});
