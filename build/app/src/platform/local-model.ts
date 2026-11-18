import type { InvokeAdapter } from "./bridge";

export interface ModelStatus {
  installed: boolean;
  modelId: "gemma-4-E2B-it";
  runtime: string;
  bytes: number;
  integrityVerified: boolean;
}

export interface ModelSuggestion {
  assetId: string;
  status: "suggested_private";
  evidenceQuote: string;
  contentHint: string | null;
  uncertainty: string | null;
  modelId: "gemma-4-E2B-it";
  runtime: string;
  backend: "cpu";
  elapsedMs: number;
  updatedAtMs: number;
}

const uuidPattern = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
function validId(id: string): void {
  if (!uuidPattern.test(id)) throw new Error("INVALID_ASSET_ID");
}

export function parseModelStatus(value: unknown): ModelStatus {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("本地模型狀態 Contract 錯誤");
  }
  const v = value as Record<string, unknown>;
  if (typeof v.installed !== "boolean" || v.modelId !== "gemma-4-E2B-it" ||
      typeof v.runtime !== "string" || !v.runtime.startsWith("litert-lm") ||
      !Number.isSafeInteger(v.bytes) || Number(v.bytes) < 0 ||
      typeof v.integrityVerified !== "boolean" ||
      (v.installed && (!v.integrityVerified || Number(v.bytes) < 2_000_000_000))) {
    throw new Error("本地模型狀態 Contract 錯誤");
  }
  return v as unknown as ModelStatus;
}

export function parseModelSuggestion(value: unknown): ModelSuggestion {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("本地模型建議 Contract 錯誤");
  }
  const v = value as Record<string, unknown>;
  if (typeof v.assetId !== "string" || !uuidPattern.test(v.assetId) ||
      v.status !== "suggested_private" || v.modelId !== "gemma-4-E2B-it" ||
      typeof v.runtime !== "string" || !v.runtime.startsWith("litert-lm") ||
      v.backend !== "cpu" ||
      typeof v.evidenceQuote !== "string" || !v.evidenceQuote.trim() ||
      (v.contentHint !== null && typeof v.contentHint !== "string") ||
      (v.uncertainty !== null && typeof v.uncertainty !== "string") ||
      !Number.isSafeInteger(v.elapsedMs) || Number(v.elapsedMs) < 0 ||
      !Number.isSafeInteger(v.updatedAtMs)) {
    throw new Error("本地模型建議 Contract 錯誤");
  }
  return v as unknown as ModelSuggestion;
}

export async function modelStatus(invoke: InvokeAdapter): Promise<ModelStatus> {
  return parseModelStatus(await invoke("local_model_status"));
}

export async function importModel(invoke: InvokeAdapter): Promise<ModelStatus> {
  return parseModelStatus(await invoke("import_local_model"));
}

export async function proposeModel(
  invoke: InvokeAdapter, id: string
): Promise<ModelSuggestion> {
  validId(id);
  return parseModelSuggestion(await invoke("local_model_propose", { id }));
}

export async function loadModelSuggestion(
  invoke: InvokeAdapter, id: string
): Promise<ModelSuggestion | null> {
  validId(id);
  const result = await invoke("load_model_suggestion", { id });
  return result === null ? null : parseModelSuggestion(result);
}
