import type { InvokeAdapter } from "./bridge";

const uuidPattern = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

export interface RegistryConcept {
  conceptId: string;
  aliases: string[];
  valueType: string;
  allowedScopes: string[];
  expiryPolicy: string;
  matchUsage: string;
  registryVersion: number;
}

export interface ParameterPreview {
  conceptId: string | null;
  valueType: "enum" | "unknown";
  valueJson: string | null;
  state: "OBSERVED" | "UNKNOWN";
  registryVersion: number;
}

export interface PrivateParameter {
  id: string;
  draftId: string;
  conceptId: string | null;
  requestedAlias: string;
  valueType: "enum" | "unknown";
  valueJson: string | null;
  rawValue: string | null;
  source: "user_explicit";
  evidenceRef: string;
  scope: "intent";
  state: "OBSERVED" | "UNKNOWN" | "CONFIRMED";
  expiryPolicy: "fixed" | "intent_lifecycle";
  expiresAtMs: number | null;
  reviewDueAtMs: null;
  visibility: "private";
  shareConsent: false;
  confirmedEventId: string | null;
  registryVersion: number;
  goalRevision: number;
  version: number;
  stale: boolean;
  matchSlotIndex: number | null;
  createdAtMs: number;
  updatedAtMs: number;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function validUuid(id: string): void {
  if (!uuidPattern.test(id)) throw new Error("INVALID_DRAFT_ID");
}

function version(value: number, min: number): void {
  if (!Number.isSafeInteger(value) || value < min) throw new Error("INVALID_VERSION");
}

export function parseRegistry(value: unknown): RegistryConcept[] {
  if (!Array.isArray(value)) throw new Error("Parameter Registry Contract 格式錯誤");
  if (!value.every((item: unknown) => {
    if (!isRecord(item)) return false;
    return typeof item.conceptId === "string" && item.conceptId.length > 0 &&
      Array.isArray(item.aliases) && item.aliases.every((v: unknown) => typeof v === "string") &&
      typeof item.valueType === "string" &&
      Array.isArray(item.allowedScopes) &&
      item.allowedScopes.every((v: unknown) => typeof v === "string") &&
      typeof item.expiryPolicy === "string" && typeof item.matchUsage === "string" &&
      Number.isSafeInteger(item.registryVersion) && Number(item.registryVersion) >= 1;
  })) throw new Error("Parameter Registry Contract 格式錯誤");
  return value as RegistryConcept[];
}

export function parsePreview(value: unknown): ParameterPreview {
  if (!isRecord(value) ||
    (value.conceptId !== null && typeof value.conceptId !== "string") ||
    !["enum", "unknown"].includes(String(value.valueType)) ||
    (value.valueJson !== null && typeof value.valueJson !== "string") ||
    !["OBSERVED", "UNKNOWN"].includes(String(value.state)) ||
    !Number.isSafeInteger(value.registryVersion) || Number(value.registryVersion) < 0) {
    throw new Error("Parameter Preview Contract 格式錯誤");
  }
  if (value.conceptId === null && (value.state !== "UNKNOWN" || value.valueJson !== null)) {
    throw new Error("Parameter Preview Contract 格式錯誤");
  }
  return value as unknown as ParameterPreview;
}

export function parsePrivateParameter(value: unknown): PrivateParameter {
  if (!isRecord(value) ||
    typeof value.id !== "string" || !uuidPattern.test(value.id) ||
    typeof value.draftId !== "string" || !uuidPattern.test(value.draftId) ||
    (value.conceptId !== null && typeof value.conceptId !== "string") ||
    typeof value.requestedAlias !== "string" || !value.requestedAlias.trim() ||
    !["enum", "unknown"].includes(String(value.valueType)) ||
    (value.valueJson !== null && typeof value.valueJson !== "string") ||
    (value.rawValue !== null && typeof value.rawValue !== "string") ||
    value.source !== "user_explicit" ||
    typeof value.evidenceRef !== "string" || !uuidPattern.test(value.evidenceRef) ||
    value.scope !== "intent" ||
    !["OBSERVED", "UNKNOWN", "CONFIRMED"].includes(String(value.state)) ||
    !["fixed", "intent_lifecycle"].includes(String(value.expiryPolicy)) ||
    (value.expiresAtMs !== null && !Number.isSafeInteger(value.expiresAtMs)) ||
    value.reviewDueAtMs !== null ||
    value.visibility !== "private" || value.shareConsent !== false ||
    (value.confirmedEventId !== null &&
      (typeof value.confirmedEventId !== "string" || !uuidPattern.test(value.confirmedEventId))) ||
    (value.state === "CONFIRMED" && value.confirmedEventId === null) ||
    (value.state !== "CONFIRMED" && value.confirmedEventId !== null) ||
    !Number.isSafeInteger(value.registryVersion) || Number(value.registryVersion) < 0 ||
    !Number.isSafeInteger(value.goalRevision) || Number(value.goalRevision) < 1 ||
    !Number.isSafeInteger(value.version) || Number(value.version) < 1 ||
    typeof value.stale !== "boolean" ||
    (value.matchSlotIndex !== null &&
      (!Number.isSafeInteger(value.matchSlotIndex) || Number(value.matchSlotIndex) < 0 ||
       Number(value.matchSlotIndex) > 7)) ||
    !Number.isSafeInteger(value.createdAtMs) || !Number.isSafeInteger(value.updatedAtMs) ||
    (value.expiryPolicy === "fixed" && (value.expiresAtMs === null || Number(value.expiresAtMs) <= 0)) ||
    (value.expiryPolicy === "intent_lifecycle" && value.expiresAtMs !== null)
  ) throw new Error("私人 Parameter Contract 格式錯誤");
  return value as unknown as PrivateParameter;
}

export async function listRegistry(invoke: InvokeAdapter): Promise<RegistryConcept[]> {
  return parseRegistry(await invoke("list_concept_registry"));
}

export async function previewConcept(
  invoke: InvokeAdapter, conceptAlias: string, value: string | null
): Promise<ParameterPreview> {
  if (!conceptAlias.trim() || new TextEncoder().encode(conceptAlias).byteLength > 160) {
    throw new Error("INVALID_PARAMETER_INPUT");
  }
  return parsePreview(await invoke("preview_parameter_concept", { conceptAlias, value }));
}

export async function listParameters(
  invoke: InvokeAdapter, draftId: string
): Promise<PrivateParameter[]> {
  validUuid(draftId);
  const response = await invoke("list_parameter_proposals", { draftId });
  if (!Array.isArray(response)) throw new Error("私人 Parameter Contract 格式錯誤");
  return response.map(parsePrivateParameter);
}

export interface SaveParameterArgs {
  draftId: string;
  expectedDraftRevision: number;
  conceptAlias: string;
  value: string | null;
  scope: "intent";
  expiryPolicy: "fixed" | "intent_lifecycle";
  expiresAtMs: number | null;
  expectedVersion: number;
}

export async function saveParameter(
  invoke: InvokeAdapter, args: SaveParameterArgs
): Promise<PrivateParameter> {
  validUuid(args.draftId);
  version(args.expectedDraftRevision, 1);
  version(args.expectedVersion, 0);
  if (!args.conceptAlias.trim() ||
    new TextEncoder().encode(args.conceptAlias).byteLength > 160 ||
    (args.value !== null && (!args.value.trim() ||
      new TextEncoder().encode(args.value).byteLength > 512))) {
    throw new Error("INVALID_PARAMETER_INPUT");
  }
  if (args.scope !== "intent") throw new Error("INVALID_SCOPE");
  if (
    (args.expiryPolicy === "fixed" &&
      (args.expiresAtMs === null || !Number.isSafeInteger(args.expiresAtMs) ||
       args.expiresAtMs <= Date.now())) ||
    (args.expiryPolicy === "intent_lifecycle" && args.expiresAtMs !== null)
  ) throw new Error("INVALID_EXPIRY");
  return parsePrivateParameter(await invoke("save_parameter_proposal", { input: args }));
}

/** Only from a real user click. No helper can fake event ID or visibility. */
export async function confirmParameter(
  invoke: InvokeAdapter, draftId: string, proposalId: string, expectedVersion: number
): Promise<PrivateParameter> {
  validUuid(draftId);
  validUuid(proposalId);
  version(expectedVersion, 1);
  return parsePrivateParameter(await invoke("confirm_private_parameter", {
    input: { draftId, proposalId, expectedVersion, confirmationAction: "confirm_parameter" },
  }));
}
