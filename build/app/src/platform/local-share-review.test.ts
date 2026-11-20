import { describe,expect,it,vi } from "vitest";
import {
  loadLocalShareReview,recordLocalShareReview,parseLocalShareReview,
  type LocalShareReview
} from "./local-share-review";

const assetId="8d1f47a6-6b58-4bb8-8291-a0e9d4ff11f3";
const draftId="937c2c93-a1c7-467a-a9c5-dc811caa6312";
const base:LocalShareReview={
  draftId,goalRevision:1,lifecycleVersion:0,
  reviewableFieldIds:["goal_text","study.jlpt_level"],
  selectedFieldIds:[],consentRecorded:false,consentVersion:0,
  canUpload:false,visibility:"private",scope:"local_preview_only",
};

describe("US-01D / B08 explicit local field review",()=>{
  it("defaults private, not uploaded or consented by preview",async()=>{
    const invoke=vi.fn().mockResolvedValue(base);
    const result=await loadLocalShareReview(invoke,assetId);
    expect(result.consentRecorded).toBe(false);
    expect(result.canUpload).toBe(false);
    expect(invoke).toHaveBeenCalledWith("load_local_share_review",{assetId});
  });
  it("user grants selected fields with an optimistic version and Rust event",async()=>{
    const next={...base,selectedFieldIds:["goal_text"],consentRecorded:true,
      consentVersion:1};
    const invoke=vi.fn().mockResolvedValue(next);
    const result=await recordLocalShareReview(invoke,assetId,base,["goal_text"],
      "approve_local_preview");
    expect(result.consentRecorded).toBe(true);
    expect(result.canUpload).toBe(false);
    expect(invoke).toHaveBeenCalledWith("record_local_share_review",{
      input:{
        assetId,expectedGoalRevision:1,expectedLifecycleVersion:0,
        expectedConsentVersion:0,selectedFieldIds:["goal_text"],
        confirmationAction:"approve_local_preview"
      }
    });
  });
  it("revoke sends empty field list and retains local audit version",async()=>{
    const before:LocalShareReview={...base,selectedFieldIds:["goal_text"],
      consentRecorded:true,consentVersion:1};
    const invoke=vi.fn().mockResolvedValue({...base,consentVersion:2});
    const after=await recordLocalShareReview(invoke,assetId,before,[],
      "revoke_local_preview");
    expect(after.consentRecorded).toBe(false);
    expect(after.consentVersion).toBe(2);
    expect(invoke).toHaveBeenCalledWith("record_local_share_review",{
      input:expect.objectContaining({
        expectedConsentVersion:1,selectedFieldIds:[],
        confirmationAction:"revoke_local_preview"
      })
    });
  });
  it("blocks unknown/private fields, duplicates, forged Cloud states",async()=>{
    const invoke=vi.fn();
    await expect(recordLocalShareReview(invoke,assetId,base,
      ["private_notes" as never],"approve_local_preview")).rejects.toThrow("B08_INVALID_FIELDS");
    await expect(recordLocalShareReview(invoke,assetId,base,
      ["goal_text","goal_text"],"approve_local_preview")).rejects.toThrow("B08_INVALID_FIELDS");
    await expect(recordLocalShareReview(invoke,assetId,base,[],
      "approve_local_preview")).rejects.toThrow("B08_INVALID_FIELDS");
    await expect(recordLocalShareReview(invoke,assetId,base,["goal_text"],
      "revoke_local_preview")).rejects.toThrow("B08_INVALID_FIELDS");
    expect(invoke).not.toHaveBeenCalled();
    for(const bad of [
      {...base,canUpload:true},
      {...base,scope:"cloud_publish"},
      {...base,visibility:"public"},
      {...base,reviewableFieldIds:["original_image"]},
      {...base,selectedFieldIds:["goal_text"],consentRecorded:false},
      {...base,selectedFieldIds:[],consentRecorded:true},
      {...base,goalRevision:0},
      {...base,consentVersion:-1},
    ]) expect(()=>parseLocalShareReview(bad)).toThrow("Contract");
  });
  it("rejects stale/fake server responses instead of claiming consent",async()=>{
    const invoke=vi.fn().mockRejectedValue("B08_STALE_REVIEW");
    await expect(recordLocalShareReview(invoke,assetId,base,["goal_text"],
      "approve_local_preview")).rejects.toBe("B08_STALE_REVIEW");
    await expect(loadLocalShareReview(vi.fn().mockResolvedValue({
      ...base,selectedFieldIds:["study.jlpt_level"],consentRecorded:true,
      reviewableFieldIds:["goal_text"]
    }),assetId)).rejects.toThrow("Contract");
  });
});
