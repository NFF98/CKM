import type { InvokeAdapter } from "./bridge";
import { MAX_TEXT_BYTES } from "./local-drops";

export interface OcrSegment {
  text: string;
  bounds: [number, number, number, number];
}

export interface OcrRecord {
  assetId: string;
  status: "not_run" | "recognized" | "unknown";
  rawText: string | null;
  correctedText: string | null;
  engine: string | null;
  engineVersion: string | null;
  segments: OcrSegment[];
  elapsedMs: number | null;
  updatedAtMs: number | null;
}

const uuidPattern = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

function validateId(id: string): void {
  if (!uuidPattern.test(id)) throw new Error("INVALID_ASSET_ID");
}

export function parseOcrRecord(value: unknown): OcrRecord {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("本地 OCR Contract 格式錯誤");
  }
  const record = value as Record<string, unknown>;
  const segments = record.segments;
  if (
    typeof record.assetId !== "string" ||
    !uuidPattern.test(record.assetId) ||
    !["not_run", "recognized", "unknown"].includes(String(record.status)) ||
    (record.rawText !== null && typeof record.rawText !== "string") ||
    (record.correctedText !== null && typeof record.correctedText !== "string") ||
    (record.engine !== null && typeof record.engine !== "string") ||
    (record.engineVersion !== null && typeof record.engineVersion !== "string") ||
    (record.elapsedMs !== null && (!Number.isSafeInteger(record.elapsedMs) || Number(record.elapsedMs) < 0)) ||
    (record.updatedAtMs !== null && !Number.isSafeInteger(record.updatedAtMs)) ||
    !Array.isArray(segments) ||
    segments.length > 500 ||
    !segments.every((segment: unknown) => {
      if (!segment || typeof segment !== "object") return false;
      const s = segment as Record<string, unknown>;
      return typeof s.text === "string" && Array.isArray(s.bounds) &&
        s.bounds.length === 4 && s.bounds.every((n: unknown) => Number.isSafeInteger(n));
    })
  ) {
    throw new Error("本地 OCR Contract 格式錯誤");
  }
  return record as unknown as OcrRecord;
}

export async function recognizeOcr(invoke: InvokeAdapter, id: string): Promise<OcrRecord> {
  validateId(id);
  return parseOcrRecord(await invoke("local_ocr_recognize", { id }));
}

export async function loadOcr(invoke: InvokeAdapter, id: string): Promise<OcrRecord> {
  validateId(id);
  return parseOcrRecord(await invoke("load_ocr", { id }));
}

export async function correctOcr(
  invoke: InvokeAdapter, id: string, correctedText: string
): Promise<OcrRecord> {
  validateId(id);
  if (!correctedText.trim()) throw new Error("EMPTY_TEXT");
  if (new TextEncoder().encode(correctedText).byteLength > MAX_TEXT_BYTES) {
    throw new Error("TEXT_TOO_LARGE");
  }
  return parseOcrRecord(await invoke("correct_ocr", { id, correctedText }));
}
