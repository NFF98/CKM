import type { InvokeAdapter } from "./bridge";

export interface ModelStatus {
  installed: boolean;
  modelId: "gemma-4-E2B-it";
  runtime: string;
  bytes: number;
  integrityVerified: boolean;
  downloadState: "idle" | "waiting_wifi" | "downloading" | "verifying" |
    "ready" | "paused" | "error" | "unavailable";
  downloadedBytes: number;
  totalBytes: number;
  downloadError: string | null;
}

export interface PrivateIntentCandidate {
  domain: "study" | "fashion" | "car" | "other" | "unknown";
  goalText: string;
  evidenceQuote: string;
}

export interface PendingIntentQuestion {
  ordinal: 1 | 2;
  text: string;
}

export interface ModelSuggestion {
  assetId: string;
  status: "suggested_private";
  evidenceQuote: string;
  contentHint: string | null;
  uncertainty: string | null;
  /** AI can suggest but NEVER confirm, publish, match or consent. */
  intentCandidate: PrivateIntentCandidate | null;
  /** Multiple evidence-checked, still private hypotheses (max 3). Legacy first item kept. */
  intentCandidates: PrivateIntentCandidate[];
  pendingQuestion: PendingIntentQuestion | null;
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
      !["idle", "waiting_wifi", "downloading", "verifying", "ready", "paused", "error", "unavailable"]
        .includes(String(v.downloadState)) ||
      !Number.isSafeInteger(v.downloadedBytes) ||
      !Number.isSafeInteger(v.totalBytes) ||
      Number(v.totalBytes) !== 2_588_147_712 ||
      Number(v.downloadedBytes) < 0 ||
      Number(v.downloadedBytes) > Number(v.totalBytes) ||
      (v.downloadError !== null && typeof v.downloadError !== "string") ||
      (v.installed && (!v.integrityVerified ||
        Number(v.bytes) !== Number(v.totalBytes) ||
        v.downloadState !== "ready")) ||
      (!v.installed && (v.integrityVerified || Number(v.bytes) !== 0)) ||
      (v.downloadState === "ready" && !v.installed)) {
    throw new Error("本地模型狀態 Contract 錯誤");
  }
  return v as unknown as ModelStatus;
}

/** Reject fabricated approval fields and malformed optional candidate objects. */
export function validPrivateIntentCandidate(v: unknown): v is PrivateIntentCandidate | null {
  if (v === null) return true;
  if (!v || typeof v !== "object" || Array.isArray(v)) return false;
  const data = v as Record<string, unknown>;
  if (Object.keys(data).sort().join(",") !== "domain,evidenceQuote,goalText") return false;
  return ["study", "fashion", "car", "other", "unknown"].includes(String(data.domain)) &&
    typeof data.goalText === "string" && data.goalText.trim().length > 0 &&
    data.goalText.length <= 500 && new TextEncoder().encode(data.goalText).byteLength <= 2048 &&
    typeof data.evidenceQuote === "string" && data.evidenceQuote.trim().length > 0 &&
    [...data.evidenceQuote].length <= 80;
}

function validPendingQuestion(value: unknown): value is PendingIntentQuestion | null {
  if (value === null) return true;
  if (!value || typeof value !== "object" || Array.isArray(value)) return false;
  const v = value as Record<string, unknown>;
  return Object.keys(v).sort().join(",") === "ordinal,text" &&
    (v.ordinal === 1 || v.ordinal === 2) &&
    typeof v.text === "string" && Boolean(v.text.trim()) &&
    [...v.text].length <= 160 && new TextEncoder().encode(v.text).length <= 512;
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
      !validPrivateIntentCandidate(v.intentCandidate) ||
      !Array.isArray(v.intentCandidates) || v.intentCandidates.length > 3 ||
      !v.intentCandidates.every(candidate =>
        candidate !== null && validPrivateIntentCandidate(candidate)) ||
      (v.intentCandidates.length === 0 && v.intentCandidate !== null) ||
      (v.intentCandidates.length > 0 &&
       JSON.stringify(v.intentCandidates[0]) !== JSON.stringify(v.intentCandidate)) ||
      !validPendingQuestion(v.pendingQuestion) ||
      (v.intentCandidate !== null && v.pendingQuestion !== null) ||
      !Number.isSafeInteger(v.elapsedMs) || Number(v.elapsedMs) < 0 ||
      !Number.isSafeInteger(v.updatedAtMs)) {
    throw new Error("本地模型建議 Contract 錯誤");
  }
  return v as unknown as ModelSuggestion;
}

export async function modelStatus(invoke: InvokeAdapter): Promise<ModelStatus> {
  return parseModelStatus(await invoke("local_model_status"));
}

/** First launch automatically initiates unmetered download. Returns immediately. */
export async function prepareModel(
  invoke: InvokeAdapter, allowMetered = false
): Promise<ModelStatus> {
  return parseModelStatus(await invoke("prepare_local_model", { allowMetered }));
}

/** User may pause a large transfer without deleting the verified partial bytes. */
export async function pauseModelDownload(invoke: InvokeAdapter): Promise<ModelStatus> {
  return parseModelStatus(await invoke("pause_local_model_download"));
}

/** Deprecated manual fallback, not exposed in the consumer interface. */
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

/** A B09 answer is explicitly typed by the person, never by the model. */
export async function answerIntentQuestion(
  invoke: InvokeAdapter, assetId: string, ordinal: 1 | 2, answer: string
): Promise<void> {
  validId(assetId);
  if ((ordinal !== 1 && ordinal !== 2) || !answer.trim() ||
      [...answer].length > 500 ||
      new TextEncoder().encode(answer).length > 2048) {
    throw new Error("B09_INVALID_ANSWER");
  }
  await invoke("answer_intent_question", { assetId, ordinal, answer: answer.trim() });
}
