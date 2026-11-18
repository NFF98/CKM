import { describe, it, expect, vi } from "vitest";
import { confirmPrivateGoal, loadPrivateGoalDraft, parsePrivateGoalDraft } from "./private-goal";

const assetId = "2bc595b3-a0e4-4254-bc8f-200b1b0f19a2";
const draftId = "f126cb5c-dc54-4666-b460-3b2e7b41ca13";
const eventId = "9891d584-30b4-4f58-98a2-4c180104bb60";
const draft = {
  id: draftId, assetId, goalText: "我要找學伴練習 JLPT N2",
  goalState: "confirmed_by_user", intentState: "draft_private",
  visibility: "private", shareConsent: false, publishable: false,
  confirmedEventId: eventId, sourceSha256: "a".repeat(64),
  revision: 1, createdAtMs: 1794890000000, updatedAtMs: 1794890000000,
};

describe("S1-AC04/B04：Content 和 Goal 分離", () => {
  it("只有 Drop 日文字、餐廳廣告：查詢回 null，不會生成 Goal", async () => {
    const invoke = vi.fn().mockResolvedValue(null);
    await expect(loadPrivateGoalDraft(invoke, assetId)).resolves.toBeNull();
    expect(invoke).toHaveBeenCalledExactlyOnceWith("load_private_goal_draft", { assetId });
  });

  it("使用者明確確認後才傳入一次 Rust Command，事件 ID 只能由 Rust 建立", async () => {
    const invoke = vi.fn().mockResolvedValue(draft);
    await expect(confirmPrivateGoal(invoke, assetId, " 我要找學伴練習 JLPT N2 ", 0))
      .resolves.toEqual(draft);
    expect(invoke).toHaveBeenCalledExactlyOnceWith("confirm_private_goal", {
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

  it("原生儲存失敗不能假回覆確認成功", async () => {
    const invoke = vi.fn().mockRejectedValue("DATABASE_WRITE_FAILED");
    await expect(confirmPrivateGoal(invoke, assetId, "我想找學伴", 0))
      .rejects.toBe("DATABASE_WRITE_FAILED");
    expect(invoke).toHaveBeenCalledTimes(1);
  });
});
