import type { InvokeAdapter } from "./bridge";

export const MAX_IMAGE_BYTES = 8 * 1024 * 1024; // B02 暫定安全上限；F-01 未定
export const MAX_TEXT_BYTES = 64 * 1024;

export interface DropSummary {
  id: string;
  kind: "text" | "image";
  source: "text" | "picker" | "camera" | "drop";
  fileName: string | null;
  mimeType: string;
  byteLength: number;
  createdAtMs: number;
  textExcerpt: string | null;
  isDuplicate: boolean;
}

export interface DropDetail {
  summary: DropSummary;
  text: string | null;
  dataBase64: string | null;
}

export type ImageSource = "picker" | "camera" | "drop";

function recordOf(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("Rust 本地儲存回傳格式錯誤");
  }
  return value as Record<string, unknown>;
}

export function parseDropSummary(value: unknown): DropSummary {
  const report = recordOf(value);
  if (
    typeof report.id !== "string" ||
    !/^[0-9a-f-]{36}$/.test(report.id) ||
    (report.kind !== "text" && report.kind !== "image") ||
    !["text", "picker", "camera", "drop"].includes(String(report.source)) ||
    (report.fileName !== null && typeof report.fileName !== "string") ||
    typeof report.mimeType !== "string" ||
    !Number.isSafeInteger(report.byteLength) || Number(report.byteLength) < 1 ||
    !Number.isSafeInteger(report.createdAtMs) ||
    (report.textExcerpt !== null && typeof report.textExcerpt !== "string") ||
    typeof report.isDuplicate !== "boolean"
  ) {
    throw new Error("Rust 本地儲存摘要 Contract 錯誤");
  }
  return report as unknown as DropSummary;
}

export function parseDropDetail(value: unknown): DropDetail {
  const report = recordOf(value);
  const summary = parseDropSummary(report.summary);
  if (
    (report.text !== null && typeof report.text !== "string") ||
    (report.dataBase64 !== null && typeof report.dataBase64 !== "string") ||
    (summary.kind === "text" && (report.text === null || report.dataBase64 !== null)) ||
    (summary.kind === "image" && (report.dataBase64 === null || report.text !== null))
  ) {
    throw new Error("Rust 本地儲存內容 Contract 錯誤");
  }
  return { summary, text: report.text as string | null,
    dataBase64: report.dataBase64 as string | null };
}

export async function saveText(invoke: InvokeAdapter, text: string): Promise<DropSummary> {
  if (!text.trim()) throw new Error("EMPTY_TEXT");
  if (new TextEncoder().encode(text).byteLength > MAX_TEXT_BYTES) {
    throw new Error("TEXT_TOO_LARGE");
  }
  return parseDropSummary(await invoke("save_drop", {
    input: { kind: "text", source: "text", text }
  }));
}

export async function saveImage(
  invoke: InvokeAdapter,
  fileName: string,
  mimeType: string,
  dataBase64: string,
  source: ImageSource
): Promise<DropSummary> {
  if (!["picker", "camera", "drop"].includes(source)) {
    throw new Error("INVALID_SOURCE");
  }
  if (dataBase64.length > Math.ceil(MAX_IMAGE_BYTES / 3) * 4 + 8) {
    throw new Error("IMAGE_TOO_LARGE");
  }
  return parseDropSummary(await invoke("save_drop", {
    input: {
      kind: "image",
      source,
      fileName,
      mimeType,
      base64: dataBase64,
    },
  }));
}

export async function listDrops(invoke: InvokeAdapter): Promise<DropSummary[]> {
  const result = await invoke("list_drops");
  if (!Array.isArray(result)) throw new Error("本地資料清單格式錯誤");
  return result.map(parseDropSummary);
}

export async function loadDrop(invoke: InvokeAdapter, id: string): Promise<DropDetail> {
  return parseDropDetail(await invoke("load_drop", { id }));
}
