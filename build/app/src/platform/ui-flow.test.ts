import { describe, expect, it } from "vitest";
import {
  ASSET_STEPS, classifyAssetPanel, moveAssetStep, modelPanelLocation,
  assetStepAvailable,
} from "./ui-flow";

describe("CAKAMI 4-step mobile GUI", () => {
  it("shows exactly four focused steps in order", () => {
    expect(ASSET_STEPS).toEqual(["source", "understand", "goal", "conditions"]);
  });
  it("keeps the original photo and OCR on the source step", () => {
    expect(classifyAssetPanel(["asset-source-section"])).toBe("source");
    expect(classifyAssetPanel(["ocr-panel"])).toBe("source");
  });
  it("keeps AI, Goal, and conditions on separate steps", () => {
    expect(classifyAssetPanel(["model-panel"])).toBe("understand");
    expect(classifyAssetPanel(["goal-panel"])).toBe("goal");
    expect(classifyAssetPanel(["parameter-panel"])).toBe("conditions");
  });
  it("has predictable next and previous navigation, with no wrapping", () => {
    expect(moveAssetStep("source", -1)).toBe("source");
    expect(moveAssetStep("source", 1)).toBe("understand");
    expect(moveAssetStep("understand", 1)).toBe("goal");
    expect(moveAssetStep("goal", 1)).toBe("conditions");
    expect(moveAssetStep("conditions", 1)).toBe("conditions");
    expect(moveAssetStep("conditions", -1)).toBe("goal");
  });
  it("moves verified E2B to device page after first setup", () => {
    expect(modelPanelLocation(true, true)).toBe("device");
    expect(modelPanelLocation(false, false)).toBe("onboarding");
    expect(modelPanelLocation(true, false)).toBe("onboarding");
  });
  it("only opens meaningful conditions when Goal is confirmed", () => {
    expect(assetStepAvailable("conditions", false)).toBe(false);
    expect(assetStepAvailable("conditions", true)).toBe(true);
    expect(assetStepAvailable("goal", false)).toBe(true);
  });
});
