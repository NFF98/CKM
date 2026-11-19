import { describe, it, expect, vi } from "vitest";
import {
  confirmPrivateGoal, loadPrivateGoalDraft, parsePrivateGoalDraft,
  updatePrivateGoalLifecycle, previewPrivateGoalShare
} from "./private-goal";

const assetId = "2bc595b3-a0e4-4254-bc8f-200b1b0f19a2";
const draftId = "f126cb5c-dc54-4666-b460-3b2e7b41ca13";
const eventId = "9891d584-30b4-4f58-98a2-4c180104bb60";
const draft = {
  id: draftId, assetId, goalText: "我要找學伴練習 JLPT N2",
  goalState: "confirmed_by_user", intentState: "draft_private",
  visibility: "private", shareConsent: false, publishable: false,
  confirmedEventId: eventId, sourceSha256: "a".repeat(64),
  lifecycleState: "active_private", expiresAtMs: null, lifecycleVersion: 0,
  revision: 1, createdAtMs: 1794890000000, updatedAtMs: 1794890000000,
};

describe("S1-AC04/B04：Content 和 Goal 分離", () => {
  it("只有 Drop 日文字、餐廳廣告：查詢回 null，不會生成 Goal", async () => {
    const invoke = vi.fn().mockResolvedValue(null);
    await expect(loadPrivateGoalDraft(invoke, assetId)).resolves.toBeNull();
    expect(invoke).toHaveBeenCalledTimes(1);
    expect(invoke).toHaveBeenCalledWith("load_private_goal_draft", { assetId });
  });

  it("使用者明確確認後才傳入一次 Rust Command，事件 ID 只能由 Rust 建立", async () => {
    const invoke = vi.fn().mockResolvedValue(draft);
    await expect(confirmPrivateGoal(invoke, assetId, " 我要找學伴練習 JLPT N2 ", 0))
      .resolves.toEqual(draft);
    expect(invoke).toHaveBeenCalledTimes(1);
    expect(invoke).toHaveBeenCalledWith("confirm_private_goal", {
      input: {
        assetId,
        goalText: "我要找學伴練習 JLPT N2",
        confirmationAction: "confirm_goal",
        expectedRevision: 0,
      },
    });
  });

  it("只有 explicit confirmation 可以修訂；傳上一版本防止過期覆寫", async () => {
    const revised = { ...draft, goalText: "一起練習 N2 聽力", revision: 2 };
    const invoke = vi.fn().mockResolvedValue(revised);
    await expect(confirmPrivateGoal(invoke, assetId, revised.goalText, 1))
      .resolves.toEqual(revised);
    expect(invoke).toHaveBeenCalledWith("confirm_private_goal", {
      input: expect.objectContaining({ expectedRevision: 1 }),
    });
  });

  it("空白、無效 ID、過長草稿不進 Rust", async () => {
    const invoke = vi.fn();
    await expect(confirmPrivateGoal(invoke, assetId, "   ", 0)).rejects.toThrow("GOAL_EMPTY");
    await expect(confirmPrivateGoal(invoke, "../../x", "N2", 0)).rejects.toThrow("INVALID_ASSET_ID");
    await expect(confirmPrivateGoal(invoke, assetId, "目".repeat(600), 0))
      .rejects.toThrow("GOAL_TOO_LARGE");
    await expect(confirmPrivateGoal(invoke, assetId, "N2", -1))
      .rejects.toThrow("GOAL_REVISION_CONFLICT");
    expect(invoke).not.toHaveBeenCalled();
  });

  it("模型若偽造 publishable、shareConsent、CONFIRMED 不會被當作合法草稿", () => {
    for (const data of [
      { ...draft, publishable: true },
      { ...draft, shareConsent: true },
      { ...draft, intentState: "published" },
      { ...draft, goalState: "model_confirmed" },
      { ...draft, confirmedEventId: "fake-model-event" },
    ]) {
      expect(() => parsePrivateGoalDraft(data)).toThrow("Contract");
    }
  });

  it("Rust 回傳的私人草稿有本人確認事件與版本，不等於分享／可配對", async () => {
    const invoke = vi.fn().mockResolvedValue(draft);
    const loaded = await loadPrivateGoalDraft(invoke, assetId);
    expect(loaded?.intentState).toBe("draft_private");
    expect(loaded?.visibility).toBe("private");
    expect(loaded?.shareConsent).toBe(false);
    expect(loaded?.publishable).toBe(false);
    expect(loaded?.confirmedEventId).toBe(eventId);
  });

  it("US-01D: 私人有效期必須是本人輸入的未來具體時間", async () => {
    const expiry=Date.now()+86_400_000;
    const expected={...draft,expiresAtMs:expiry,lifecycleVersion:1};
    const invoke=vi.fn().mockResolvedValue(expected);
    const received=await updatePrivateGoalLifecycle(invoke,assetId,1,0,"set_expiry",expiry);
    expect(received.expiresAtMs).toBe(expiry);
    expect(received.shareConsent).toBe(false);
    expect(invoke).toHaveBeenCalledWith("update_private_goal_lifecycle",{
      input:{assetId,expectedRevision:1,expectedLifecycleVersion:0,
        action:"set_expiry",expiresAtMs:expiry}
    });
    await expect(updatePrivateGoalLifecycle(invoke,assetId,1,0,"set_expiry",null))
      .rejects.toThrow("LIFECYCLE_INVALID_ACTION");
    await expect(updatePrivateGoalLifecycle(invoke,assetId,1,0,"cancel_goal",expiry))
      .rejects.toThrow("LIFECYCLE_INVALID_ACTION");
    expect(invoke).toHaveBeenCalledTimes(1);
  });

  it("US-01D: 本人撤銷不會產生 Consent 或復活私人 Intent", async () => {
    const cancelled={...draft,lifecycleState:"cancelled_private",lifecycleVersion:1};
    const invoke=vi.fn().mockResolvedValue(cancelled);
    const received=await updatePrivateGoalLifecycle(invoke,assetId,1,0,"cancel_goal",null);
    expect(received.lifecycleState).toBe("cancelled_private");
    expect(received.publishable).toBe(false);
    expect(invoke).toHaveBeenCalledWith("update_private_goal_lifecycle",{
      input:{assetId,expectedRevision:1,expectedLifecycleVersion:0,
        action:"cancel_goal",expiresAtMs:null}
    });
  });

  it("US-01D: share preview 僅本地讀取，永遠不可上傳或冒充已同意", async () => {
    const payload={
      draftId,goalText:draft.goalText,lifecycleState:"active_private",
      possibleConceptIds:["study.jlpt_level"],visibility:"private",
      consentRecorded:false,canUpload:false,approvalStatus:"T01_AND_CONSENT_PENDING",
      excludedPrivateSources:["original_image","raw_ocr","clarification_answers"]
    };
    const invoke=vi.fn().mockResolvedValue(payload);
    const preview=await previewPrivateGoalShare(invoke,assetId);
    expect(preview.canUpload).toBe(false);
    expect(invoke).toHaveBeenCalledWith("preview_private_goal_share",{assetId});
    expect(invoke).toHaveBeenCalledTimes(1);
    for(const bad of [
      {...payload,canUpload:true},
      {...payload,consentRecorded:true},
      {...payload,possibleConceptIds:["N2",123]},
      {...payload,visibility:"public"},
    ]) expect(()=>previewPrivateGoalShare(vi.fn().mockResolvedValue(bad),assetId))
      .rejects.toThrow("Contract");
  });

  it("原生儲存失敗不能假回覆確認成功", async () => {
    const invoke = vi.fn().mockRejectedValue("DATABASE_WRITE_FAILED");
    await expect(confirmPrivateGoal(invoke, assetId, "我想找學伴", 0))
      .rejects.toBe("DATABASE_WRITE_FAILED");
    expect(invoke).toHaveBeenCalledTimes(1);
  });
});
