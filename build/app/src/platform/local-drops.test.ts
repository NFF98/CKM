import { describe, expect, it, vi } from "vitest";
import { listDrops, loadDrop, MAX_IMAGE_BYTES, parseDropSummary, saveImage, saveText } from "./local-drops";

const asset = {
  id: "2bc595b3-a0e4-4254-bc8f-200b1b0f19a2",
  kind: "text",
  source: "text",
  fileName: null,
  mimeType: "text/plain",
  byteLength: 10,
  createdAtMs: 1794890000000,
  textExcerpt: "パクる",
  isDuplicate: false
} as const;

describe("S1-AC02：本地 Drop Contract", () => {
  it("文字只能呼叫 Rust 儲存，不經網路", async () => {
    const invoke = vi.fn().mockResolvedValue(asset);
    await expect(saveText(invoke, "パクる")).resolves.toEqual(asset);
    expect(invoke).toHaveBeenCalledWith("save_drop", {
      input: { kind: "text", source: "text", text: "パクる" }
    });
  });

  it("空白與過大文字在前端先拒絕", async () => {
    const invoke = vi.fn();
    await expect(saveText(invoke, "   ")).rejects.toThrow("EMPTY_TEXT");
    await expect(saveText(invoke, "日".repeat(23000))).rejects.toThrow("TEXT_TOO_LARGE");
    expect(invoke).not.toHaveBeenCalled();
  });

  it("圖片傳遞來源與純 base64，不傳私人路徑", async () => {
    const summary = { ...asset, kind: "image", source: "camera", fileName: "N2.png",
      mimeType: "image/png", textExcerpt: null };
    const invoke = vi.fn().mockResolvedValue(summary);
    await expect(saveImage(invoke, "N2.png", "image/png", "AAAA", "camera"))
      .resolves.toEqual(summary);
    expect(invoke).toHaveBeenCalledWith("save_drop", {
      input: { kind: "image", source: "camera", fileName: "N2.png",
        mimeType: "image/png", base64: "AAAA" },
    });
  });

  it("巨大圖片在前端拒絕", async () => {
    const invoke = vi.fn();
    await expect(saveImage(invoke, "huge.png", "image/png",
      "A".repeat(Math.ceil(MAX_IMAGE_BYTES / 3) * 4 + 9), "picker"))
      .rejects.toThrow("IMAGE_TOO_LARGE");
    expect(invoke).not.toHaveBeenCalled();
  });

  it("Rust 回應不合法不會假顯示儲存成功", () => {
    expect(() => parseDropSummary({ ...asset, byteLength: -1 })).toThrow();
    expect(() => parseDropSummary({ ...asset, id: "../../secrets" })).toThrow();
  });

  it("跨次開啟的清單只從 Rust SQLite 重讀", async () => {
    const invoke = vi.fn().mockResolvedValue([asset]);
    await expect(listDrops(invoke)).resolves.toEqual([asset]);
    expect(invoke).toHaveBeenCalledWith("list_drops");
  });

  it("可按 ID 讀取全文，且不把原圖拿來當 OCR 結果", async () => {
    const invoke = vi.fn().mockResolvedValue({
      summary: asset, text: "パクる", dataBase64: null,
    });
    await expect(loadDrop(invoke, asset.id)).resolves.toEqual({
      summary: asset, text: "パクる", dataBase64: null,
    });
    expect(invoke).toHaveBeenCalledWith("load_drop", { id: asset.id });
  });

  it("原圖內容 Contract 不接受用文字冒充圖片", async () => {
    const invoke = vi.fn().mockResolvedValue({
      summary: { ...asset, kind: "image", mimeType: "image/png" },
      text: "fake OCR", dataBase64: null
    });
    await expect(loadDrop(invoke, asset.id)).rejects.toThrow();
  });
});
