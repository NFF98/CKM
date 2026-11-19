import { describe, it, expect, vi } from "vitest";
import {
  confirmParameter, listParameters, listRegistry, parsePrivateParameter,
  previewConcept, saveParameter, type PrivateParameter
} from "./private-parameters";

const draftId = "f126cb5c-dc54-4666-b460-3b2e7b41ca13";
const id = "2040f2fe-803a-4318-8814-5f264e036feb";
const evidenceRef = "9891d584-30b4-4f58-98a2-4c180104bb60";

const observed: PrivateParameter = {
  id, draftId, conceptId: "study.jlpt_level", requestedAlias: "JLPT N2",
  valueType: "enum", valueJson: '"N2"', rawValue: "N2",
  source: "user_explicit", evidenceRef, scope: "intent",
  state: "OBSERVED", expiryPolicy: "intent_lifecycle",
  expiresAtMs: null, reviewDueAtMs: null, visibility: "private",
  shareConsent: false, confirmedEventId: null, registryVersion: 1,
  goalRevision: 1, version: 1, stale: false, matchSlotIndex: null,
  createdAtMs: 1794890000000, updatedAtMs: 1794890000000,
};

describe("B05 Private Parameter Registry Contract", () => {
  it("Registry 僅列出經已知規格批准的概念", async () => {
    const registry = [{
      conceptId:"study.jlpt_level",aliases:["N2","JLPT N2","日語二級"],
      valueType:"enum",allowedScopes:["intent"],expiryPolicy:"intent_lifecycle",
      matchUsage:"pending_T01_review",registryVersion:1
    }];
    const invoke=vi.fn().mockResolvedValue(registry);
    expect((await listRegistry(invoke))[0].conceptId).toBe("study.jlpt_level");
    expect(invoke).toHaveBeenCalledWith("list_concept_registry");
  });

  it("N2 標準化預覽只讀 Rust，不靠 TS 猜測", async () => {
    const invoke=vi.fn().mockResolvedValue({
      conceptId:"study.jlpt_level",valueType:"enum",valueJson:'"N2"',
      state:"OBSERVED",registryVersion:1
    });
    expect((await previewConcept(invoke,"日語二級",null)).conceptId).toBe("study.jlpt_level");
    expect(invoke).toHaveBeenCalledWith("preview_parameter_concept",{
      conceptAlias:"日語二級",value:null
    });
  });

  it("只能為已確認 Goal 的私人 draft 送出 user proposal，沒有 source／consent 欄位", async () => {
    const invoke=vi.fn().mockResolvedValue(observed);
    const args={
      draftId,expectedDraftRevision:1,conceptAlias:"JLPT N2",value:null,
      scope:"intent" as const,expiryPolicy:"intent_lifecycle" as const,
      expiresAtMs:null,expectedVersion:0,
    };
    expect((await saveParameter(invoke,args)).state).toBe("OBSERVED");
    expect(invoke).toHaveBeenCalledWith("save_parameter_proposal",{input:args});
  });

  it("讀取多筆本地候選，UNKNOWN 沒有 confirmedEventId", async () => {
    const unknown={...observed,conceptId:null,valueType:"unknown",
      valueJson:null,state:"UNKNOWN",requestedAlias:"new.concept",registryVersion:0};
    const invoke=vi.fn().mockResolvedValue([observed,unknown]);
    const data=await listParameters(invoke,draftId);
    expect(data).toHaveLength(2);
    expect(data[1].conceptId).toBeNull();
  });

  it("人工確認命令只帶 UUID 與版本，event ID 由 Rust 生成", async () => {
    const confirmed={...observed,state:"CONFIRMED",version:2,
      confirmedEventId:"995f67bb-ddde-4c74-9e48-bc2c5e14455e",matchSlotIndex:0};
    const invoke=vi.fn().mockResolvedValue(confirmed);
    expect((await confirmParameter(invoke,draftId,id,1)).state).toBe("CONFIRMED");
    expect(invoke).toHaveBeenCalledWith("confirm_private_parameter",{
      input:{draftId,proposalId:id,expectedVersion:1,
        confirmationAction:"confirm_parameter"}
    });
  });

  it("反對 AI 假確認、公開或不受控 scope 的 Rust 回傳", () => {
    for(const invalid of [
      {...observed,source:"model_inferred"},
      {...observed,visibility:"published"},
      {...observed,shareConsent:true},
      {...observed,scope:"profile"},
      {...observed,state:"CONFIRMED"},
      {...observed,confirmedEventId:"fake"},
      {...observed,matchSlotIndex:8},
      {...observed,matchSlotIndex:-1},
    ]) {
      expect(()=>parsePrivateParameter(invalid)).toThrow("Contract");
    }
  });

  it("固定時效必須明確時間，不能把週六自動視作每週六", async () => {
    const invoke=vi.fn();
    const fixed={
      draftId,expectedDraftRevision:1,conceptAlias:"週六有空",value:null,
      scope:"intent" as const,expiryPolicy:"fixed" as const,
      expiresAtMs:null,expectedVersion:0,
    };
    await expect(saveParameter(invoke,fixed)).rejects.toThrow("INVALID_EXPIRY");
    expect(invoke).not.toHaveBeenCalled();
  });

  it("拒絕壞 UUID、壞版本、過長別名，不做任何 Rust 寫入", async () => {
    const invoke=vi.fn();
    await expect(confirmParameter(invoke,"../../db",id,1)).rejects.toThrow("INVALID_DRAFT_ID");
    await expect(confirmParameter(invoke,draftId,id,0)).rejects.toThrow("INVALID_VERSION");
    await expect(saveParameter(invoke,{
      draftId,expectedDraftRevision:1,conceptAlias:"a".repeat(200),value:null,
      scope:"intent",expiryPolicy:"intent_lifecycle",expiresAtMs:null,expectedVersion:0,
    })).rejects.toThrow("INVALID_PARAMETER_INPUT");
    expect(invoke).not.toHaveBeenCalled();
  });

  it("Rust 失敗時不假稱參數已確認", async () => {
    const invoke=vi.fn().mockRejectedValue("PARAMETER_VERSION_CONFLICT");
    await expect(confirmParameter(invoke,draftId,id,1))
      .rejects.toBe("PARAMETER_VERSION_CONFLICT");
    expect(invoke).toHaveBeenCalledTimes(1);
  });
});
