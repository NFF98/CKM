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
