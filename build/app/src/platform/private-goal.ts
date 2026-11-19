import type { InvokeAdapter } from "./bridge";

/** B04：這是用戶已確認的私有 Goal 草稿，不是可發布的配對 Intent。 */
export interface PrivateGoalDraft {
  id: string;
  assetId: string;
  goalText: string;
  goalState: "confirmed_by_user";
  intentState: "draft_private";
  visibility: "private";
  shareConsent: false;
  publishable: false;
  confirmedEventId: string;
  sourceSha256: string;
  lifecycleState: "active_private" | "cancelled_private" | "expired_private";
  expiresAtMs: number | null;
  lifecycleVersion: number;
  revision: number;
  createdAtMs: number;
  updatedAtMs: number;
}

const uuid = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
const sha256 = /^[0-9a-f]{64}$/i;

function validateAssetId(assetId: string): void {
  if (!uuid.test(assetId)) throw new Error("INVALID_ASSET_ID");
}

export function parsePrivateGoalDraft(value: unknown): PrivateGoalDraft {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("私人 Goal Contract 格式錯誤");
  }
  const v = value as Record<string, unknown>;
  if (
    typeof v.id !== "string" || !uuid.test(v.id) ||
    typeof v.assetId !== "string" || !uuid.test(v.assetId) ||
    typeof v.goalText !== "string" || !v.goalText.trim() ||
    v.goalState !== "confirmed_by_user" ||
    v.intentState !== "draft_private" ||
    v.visibility !== "private" ||
    v.shareConsent !== false ||
    v.publishable !== false ||
    typeof v.confirmedEventId !== "string" || !uuid.test(v.confirmedEventId) ||
    typeof v.sourceSha256 !== "string" || !sha256.test(v.sourceSha256) ||
    !["active_private", "cancelled_private", "expired_private"].includes(String(v.lifecycleState)) ||
    (v.expiresAtMs !== null && (!Number.isSafeInteger(v.expiresAtMs) || Number(v.expiresAtMs) <= 0)) ||
    !Number.isSafeInteger(v.lifecycleVersion) || Number(v.lifecycleVersion) < 0 ||
    (v.lifecycleState === "expired_private" && v.expiresAtMs === null) ||
    !Number.isSafeInteger(v.revision) || Number(v.revision) < 1 ||
    !Number.isSafeInteger(v.createdAtMs) ||
    !Number.isSafeInteger(v.updatedAtMs)
  ) {
    throw new Error("私人 Goal Contract 格式錯誤");
  }
  return v as unknown as PrivateGoalDraft;
}

export async function loadPrivateGoalDraft(
  invoke: InvokeAdapter, assetId: string
): Promise<PrivateGoalDraft | null> {
  validateAssetId(assetId);
  const response = await invoke("load_private_goal_draft", { assetId });
  return response === null ? null : parsePrivateGoalDraft(response);
}

/**
 * Only the explicit user-confirmation interaction should call this method.
 * There are deliberately no APIs for publishing, automatic Goal creation or consent.
 * Rust generates the event ID and enforces versioned SQLite transactions.
 */
export async function confirmPrivateGoal(
  invoke: InvokeAdapter, assetId: string, goalText: string, expectedRevision: number
): Promise<PrivateGoalDraft> {
  validateAssetId(assetId);
  const trimmed = goalText.trim();
  if (!trimmed) throw new Error("GOAL_EMPTY");
  if (trimmed.includes("\0")) throw new Error("GOAL_EMPTY");
  if (new TextEncoder().encode(trimmed).byteLength > 2048 ||
      [...trimmed].length > 500) {
    throw new Error("GOAL_TOO_LARGE");
  }
  if (!Number.isSafeInteger(expectedRevision) || expectedRevision < 0) {
    throw new Error("GOAL_REVISION_CONFLICT");
  }
  return parsePrivateGoalDraft(await invoke("confirm_private_goal", {
    input: {
      assetId,
      goalText: trimmed,
      confirmationAction: "confirm_goal",
      expectedRevision,
    },
  }));
}

/** The person may set a real future deadline or cancel a local Intent. No cloud calls. */
export async function updatePrivateGoalLifecycle(
  invoke: InvokeAdapter,
  assetId: string,
  expectedRevision: number,
  expectedLifecycleVersion: number,
  action: "set_expiry" | "cancel_goal",
  expiresAtMs: number | null
): Promise<PrivateGoalDraft> {
  validateAssetId(assetId);
  if (!Number.isSafeInteger(expectedRevision) || expectedRevision < 1 ||
      !Number.isSafeInteger(expectedLifecycleVersion) || expectedLifecycleVersion < 0) {
    throw new Error("GOAL_REVISION_CONFLICT");
  }
  if ((action === "set_expiry" && (expiresAtMs === null ||
       !Number.isSafeInteger(expiresAtMs) || expiresAtMs <= Date.now())) ||
      (action === "cancel_goal" && expiresAtMs !== null)) {
    throw new Error("LIFECYCLE_INVALID_ACTION");
  }
  return parsePrivateGoalDraft(await invoke("update_private_goal_lifecycle", {
    input: {
      assetId, expectedRevision, expectedLifecycleVersion, action, expiresAtMs,
    },
  }));
}

export interface PrivateSharePreview {
  draftId: string;
  goalText: string;
  lifecycleState: "active_private" | "cancelled_private" | "expired_private";
  possibleConceptIds: string[];
  visibility: "private";
  consentRecorded: false;
  canUpload: false;
  approvalStatus: "T01_AND_CONSENT_PENDING";
  excludedPrivateSources: string[];
}

/** Read-only B08 preview. NEVER treated as Consent or an approved payload. */
export async function previewPrivateGoalShare(
  invoke: InvokeAdapter, assetId: string
): Promise<PrivateSharePreview> {
  validateAssetId(assetId);
  const result: unknown = await invoke("preview_private_goal_share", { assetId });
  if (!result || typeof result !== "object" || Array.isArray(result)) {
    throw new Error("私人 Share Preview Contract 格式錯誤");
  }
  const value=result as Record<string,unknown>;
  if (typeof value.draftId !== "string" || !uuid.test(value.draftId) ||
      typeof value.goalText !== "string" || !value.goalText.trim() ||
      !["active_private","cancelled_private","expired_private"].includes(
        String(value.lifecycleState)) ||
      !Array.isArray(value.possibleConceptIds) ||
      !value.possibleConceptIds.every((v:unknown)=>typeof v==="string") ||
      value.visibility!=="private" || value.consentRecorded!==false ||
      value.canUpload!==false || value.approvalStatus!=="T01_AND_CONSENT_PENDING" ||
      !Array.isArray(value.excludedPrivateSources) ||
      !value.excludedPrivateSources.every((v:unknown)=>typeof v==="string")) {
    throw new Error("私人 Share Preview Contract 格式錯誤");
  }
  return value as unknown as PrivateSharePreview;
}
