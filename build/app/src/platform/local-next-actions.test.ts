import { describe, expect, it } from "vitest";
import { suggestLocalNextActions } from "./local-next-actions";

describe("JLPT can produce options without guessing a confirmed Intent", () => {
  it("gives three immediate actions for the single word JLPT", () => {
    const actions = suggestLocalNextActions("JLPT");
    expect(actions.map(x => x.kind)).toEqual(["explain", "save", "draft"]);
    expect(actions.map(x => x.title)).toEqual([
      "看 JLPT 級別", "建立 JLPT 複習卡", "找人一起練日文",
    ]);
    expect(actions[0].content).toContain("級別未指定");
  });
  it("does not silently assume the user's JLPT level", () => {
    expect(suggestLocalNextActions("JLPT N2")[0].content).toContain("N2");
    expect(suggestLocalNextActions("JLPT")[2].content).toContain("目標級別由我再確認");
  });
  it("provides a practice note for Japanese and a safe generic fallback", () => {
    expect(suggestLocalNextActions("パクる")[0].kind).toBe("save");
    expect(suggestLocalNextActions("new car")[2].kind).toBe("draft");
  });
  it("never invents a goal or action when the source is empty", () => {
    expect(suggestLocalNextActions("  ")).toEqual([]);
  });
  it("does not insert any publish or consent command", () => {
    for (const text of ["JLPT", "パクる", "T-shirt 版型", "car"]) {
      const actions = suggestLocalNextActions(text);
      expect(actions).toHaveLength(3);
      expect(actions.every(x => ["explain", "save", "draft"].includes(x.kind))).toBe(true);
      expect(actions.every(x => !/consent|publish|upload/i.test(x.id))).toBe(true);
    }
  });
});
