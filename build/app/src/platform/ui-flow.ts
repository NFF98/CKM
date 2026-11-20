/** Mobile GUI navigation policy: never show all US-01 panels on one page. */
export const ASSET_STEPS = ["source", "understand", "goal", "conditions"] as const;
export type AssetStep = typeof ASSET_STEPS[number];

export const PANEL_STEP: Record<string, AssetStep> = {
  "asset-source-section": "source",
  "ocr-panel": "source",
  "model-panel": "understand",
  "goal-panel": "goal",
  "parameter-panel": "conditions",
};

export function classifyAssetPanel(classes: Iterable<string>): AssetStep {
  for (const className of classes) {
    const step = PANEL_STEP[className];
    if (step) return step;
  }
  return "source";
}

export function moveAssetStep(step: AssetStep, delta: -1 | 1): AssetStep {
  const index = ASSET_STEPS.indexOf(step);
  return ASSET_STEPS[Math.min(Math.max(0, index + delta), ASSET_STEPS.length - 1)];
}

export function modelPanelLocation(installed: boolean, integrityVerified: boolean): "device" | "onboarding" {
  // A verified E2B must never keep filling the first-use page after an APK update.
  return installed && integrityVerified ? "device" : "onboarding";
}

export function assetStepAvailable(step: AssetStep, hasGoal: boolean): boolean {
  return step !== "conditions" || hasGoal;
}
