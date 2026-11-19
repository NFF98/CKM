import type { InvokeAdapter } from "./bridge";

const uuid=/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
export type LocalShareField = "goal_text" | "study.jlpt_level";

export interface LocalShareReview {
  draftId: string;
  goalRevision: number;
  lifecycleVersion: number;
  reviewableFieldIds: LocalShareField[];
  selectedFieldIds: LocalShareField[];
  consentRecorded: boolean;
  consentVersion: number;
  canUpload: false;
  visibility: "private";
  scope: "local_preview_only";
}

const allowed=new Set<LocalShareField>(["goal_text","study.jlpt_level"]);
const isList=(value:unknown):value is LocalShareField[] =>
  Array.isArray(value) && value.every((field:unknown)=>allowed.has(field as LocalShareField))
  && new Set(value).size===value.length;

export function parseLocalShareReview(value:unknown):LocalShareReview {
  if (!value || typeof value!=="object" || Array.isArray(value))
    throw new Error("B08 本地授權 Contract 格式錯誤");
  const v=value as Record<string,unknown>;
  if (typeof v.draftId!=="string" || !uuid.test(v.draftId)
    || !Number.isSafeInteger(v.goalRevision) || Number(v.goalRevision)<1
    || !Number.isSafeInteger(v.lifecycleVersion) || Number(v.lifecycleVersion)<0
    || !Number.isSafeInteger(v.consentVersion) || Number(v.consentVersion)<0
    || !isList(v.reviewableFieldIds) || !isList(v.selectedFieldIds)
    || typeof v.consentRecorded!=="boolean"
    || (v.consentRecorded===true &&
        (v.selectedFieldIds.length===0 || v.selectedFieldIds.some(f=>!v.reviewableFieldIds.includes(f))))
    || (v.consentRecorded===false && v.selectedFieldIds.length!==0)
    || v.canUpload!==false || v.visibility!=="private" || v.scope!=="local_preview_only")
    throw new Error("B08 本地授權 Contract 格式錯誤");
  return v as unknown as LocalShareReview;
}

export async function loadLocalShareReview(invoke:InvokeAdapter,assetId:string):
  Promise<LocalShareReview> {
  if (!uuid.test(assetId)) throw new Error("INVALID_ASSET_ID");
  return parseLocalShareReview(await invoke("load_local_share_review",{assetId}));
}

/**
 * A human explicitly reviews individual fields for a LOCAL preview only.
 * Cannot grant Cloud upload; Rust owns the allowlist, version and event ledger.
 */
export async function recordLocalShareReview(
  invoke:InvokeAdapter,assetId:string,previous:LocalShareReview,
  selected:LocalShareField[],action:"approve_local_preview"|"revoke_local_preview"
):Promise<LocalShareReview> {
  if (!uuid.test(assetId) || !uuid.test(previous.draftId)) throw new Error("INVALID_ASSET_ID");
  if (!Number.isSafeInteger(previous.goalRevision) || previous.goalRevision<1
    || !Number.isSafeInteger(previous.lifecycleVersion) || previous.lifecycleVersion<0
    || !Number.isSafeInteger(previous.consentVersion) || previous.consentVersion<0)
    throw new Error("B08_STALE_REVIEW");
  if (!isList(selected) || (action==="approve_local_preview" &&
        (selected.length===0 || selected.some(f=>!previous.reviewableFieldIds.includes(f))))
    || (action==="revoke_local_preview" && selected.length!==0))
    throw new Error("B08_INVALID_FIELDS");
  return parseLocalShareReview(await invoke("record_local_share_review",{
    input:{
      assetId,
      expectedGoalRevision:previous.goalRevision,
      expectedLifecycleVersion:previous.lifecycleVersion,
      expectedConsentVersion:previous.consentVersion,
      selectedFieldIds:selected,
      confirmationAction:action,
    }
  }));
}
