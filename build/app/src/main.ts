import { invoke, isTauri } from "@tauri-apps/api/core";
import { checkBridge, type InvokeAdapter } from "./platform/bridge";
import {
  MAX_IMAGE_BYTES,
  deleteDrop,
  loadDrop,
  listDrops,
  saveImage,
  saveText,
  type ImageSource,
} from "./platform/local-drops";
import { correctOcr, loadOcr, recognizeOcr, type OcrRecord } from "./platform/local-ocr";
import {
  prepareModel,
  pauseModelDownload,
  loadModelSuggestion,
  modelStatus,
  proposeModel,
  type ModelStatus,
  type ModelSuggestion
} from "./platform/local-model";
import {
  confirmPrivateGoal,
  loadPrivateGoalDraft,
  type PrivateGoalDraft
} from "./platform/private-goal";
import {
  confirmParameter,
  listParameters,
  listRegistry,
  previewConcept,
  saveParameter,
  type PrivateParameter,
} from "./platform/private-parameters";
import "./styles.css";

type View = "drop" | "diagnostic";

const root = document.querySelector<HTMLDivElement>("#app");
if (!root) throw new Error("找不到 App Root");

root.innerHTML = `
  <div class="shell">
    <header class="top">
      <div class="brand"><span class="mark" aria-hidden="true">C</span><span>CAKAMI</span></div>
      <p class="top-caption">私人內容・本地理解・真人學伴</p>
    </header>
    <nav class="tabs" aria-label="主要導覽">
      <button class="tab active" id="tab-drop" type="button" aria-current="page">開始</button>
      <button class="tab" id="tab-diagnostic" type="button">裝置狀態</button>
    </nav>

    <main id="main-content">
      <section id="view-drop" class="view">
        <p class="eyebrow">SPRINT 1 · US-01A</p>
        <h1>從文字或圖片，開始理解你想做的事。</h1>
        <p class="lead">輸入文字或選擇圖片。Gemma 可以提出私人意圖建議，但只有你親自確認才會形成 Goal；目前不會自動找人或上傳原圖。</p>

        <section class="note" aria-labelledby="drop-heading">
          <h2 id="drop-heading">放入圖片或文字</h2>
          <div class="actions">
            <button type="button" class="primary" id="choose-image">選擇圖片</button>
            <button type="button" class="secondary" id="take-photo">拍照</button>
          </div>
          <input type="file" id="image-picker" accept="image/png,image/jpeg,image/webp" hidden aria-label="選擇私人圖片" />
          <input type="file" id="camera-picker" accept="image/png,image/jpeg,image/webp" capture="environment" hidden aria-label="手機拍照" />
          <label for="drop-text" class="field-label">或輸入任何文字（不用圖片）</label>
          <textarea id="drop-text" rows="4" placeholder="例如：我想找人一起練 JLPT N2 口說" spellcheck="false"></textarea>
          <button type="button" class="primary" id="save-text">保存並理解這段文字</button>
          <p class="fineprint">目前圖片僅支援 PNG／JPEG／WebP，單張上限暫定 8 MB；文字最多 64 KB。這是開發期安全上限，仍待正式規格確認。</p>
          <p id="save-status" role="status" aria-live="polite" class="status">尚未選擇內容</p>
        </section>

        <section aria-labelledby="recent-heading" class="recent-section">
          <div class="section-title">
            <h2 id="recent-heading">我的私人收藏</h2>
            <button type="button" class="text-action" id="reload-drops">重新整理</button>
          </div>
          <p id="recent-status" class="fineprint" role="status">正在讀取手機本地資料…</p>
          <ul id="recent-drops" class="recent-list" aria-label="私人收藏清單"></ul>
          <div id="drop-preview" class="preview" hidden aria-live="polite"></div>
        </section>

        <section class="note" aria-labelledby="model-heading">
          <h2 id="model-heading">裝置端 Gemma 4 E2B</h2>
          <p id="model-status" class="status" role="status" aria-live="polite">正在檢查本地模型…</p>
          <p class="fineprint">第一次開啟會自動在 Wi-Fi／非計量網路下載約 2.59 GB 的官方模型，完成後自動驗證並安裝；完全不用自己找檔案。只下載公開模型，私人圖片、OCR 和學習內容不會送往雲端。下載完成後可以離線推論。</p>
          <progress id="model-progress" max="100" value="0" aria-label="Gemma 模型下載進度" hidden></progress>
          <p id="model-download-percent" class="fineprint" aria-live="polite"></p>
          <div class="model-download-actions">
            <button type="button" class="secondary" id="retry-model-download" hidden>繼續／重新下載模型（Wi-Fi）</button>
            <button type="button" class="secondary" id="allow-mobile-download" hidden>改用行動網路下載 2.59 GB</button>
            <button type="button" class="secondary" id="pause-model-download" hidden>暫停下載</button>
          </div>
        </section>

        <dialog id="parameter-dialog" class="confirm-dialog"
          aria-labelledby="parameter-dialog-title" aria-describedby="parameter-dialog-explanation">
          <h2 id="parameter-dialog-title">本人確認這個條件？</h2>
          <p id="parameter-dialog-explanation">這只會建立手機裡的私人參數確認事件；不是發布、不是分享同意，也不會立即搜尋學伴。</p>
          <p id="parameter-confirm-summary" class="goal-confirm-summary"></p>
          <p id="parameter-confirm-status" class="status" role="status" aria-live="polite"></p>
          <div class="delete-dialog-actions">
            <button id="parameter-cancel" type="button" class="secondary">返回修改</button>
            <button id="parameter-commit" type="button" class="primary">確認此私人條件</button>
          </div>
        </dialog>
        <dialog id="goal-dialog" class="confirm-dialog" aria-labelledby="goal-dialog-title" aria-describedby="goal-dialog-explanation">
          <h2 id="goal-dialog-title">確認這是你真正想做的事？</h2>
          <p id="goal-dialog-explanation">只有你按下「確認我的 Goal」，Rust 才會建立或更新手機裡的私人意圖草稿與本人確認事件。這不會公開、尋找學伴或傳送資料。</p>
          <p id="goal-confirm-summary" class="goal-confirm-summary"></p>
          <p id="goal-confirm-status" role="status" aria-live="polite" class="status"></p>
          <div class="delete-dialog-actions">
            <button type="button" class="secondary" id="goal-cancel">返回修改</button>
            <button type="button" class="primary" id="goal-commit">確認我的 Goal（只存私人草稿）</button>
          </div>
        </dialog>
        <dialog id="delete-dialog" class="confirm-dialog" aria-labelledby="delete-title" aria-describedby="delete-description">
          <h2 id="delete-title">確定要刪除這筆收藏？</h2>
          <p id="delete-description">這項操作無法復原。確認後，手機內的收藏紀錄與對應私人圖片會一起刪除，不會影響原本相簿中的照片。</p>
          <p id="delete-item" class="delete-item"></p>
          <p id="delete-status" class="status" role="status" aria-live="polite"></p>
          <div class="delete-dialog-actions">
            <button type="button" class="secondary" id="cancel-delete">取消</button>
            <button type="button" class="danger" id="confirm-delete">確認刪除</button>
          </div>
        </dialog>

        <div class="note boundary">
          <strong>US-01｜內容不等於本人意圖</strong>
          <p>本地 Gemma 可以提出私人 Goal 候選，你可以採用、修改或只收藏；只有你按下確認，Rust 才保存私人草稿。目前不會公開、配對或上傳。</p>
        </div>
        <button type="button" class="secondary" id="go-diagnostic">檢查裝置端 Rust 連線</button>
      </section>

      <section id="view-diagnostic" class="view" hidden>
        <p class="eyebrow">B01 · 連線診斷</p>
        <h1>裝置狀態</h1>
        <p class="lead">這項檢查呼叫手機 App 內的 Rust Command，不使用假的 API 回應。</p>
        <div class="diagnostic-panel">
          <p class="diagnostic-title">Rust Bridge</p>
          <p id="bridge-status" role="status" aria-live="polite">尚未執行檢查</p>
          <p id="bridge-detail" class="detail">尚未檢查手機內的模型狀態。</p>
        </div>
        <button type="button" class="primary" id="check-bridge">執行連線檢查</button>
        <button type="button" class="secondary" id="back-home">返回開始</button>
      </section>
    </main>
    <footer class="footer">CAKAMI · Local-first · 僅限 18 歲以上學習者</footer>
  </div>
`;

const get = <T extends HTMLElement>(selector: string): T => {
  const element = document.querySelector<T>(selector);
  if (!element) throw new Error(`找不到 GUI 元件：${selector}`);
  return element;
};

const dropView = get<HTMLElement>("#view-drop");
const diagnosticView = get<HTMLElement>("#view-diagnostic");
const tabDrop = get<HTMLButtonElement>("#tab-drop");
const tabDiagnostic = get<HTMLButtonElement>("#tab-diagnostic");
const status = get<HTMLElement>("#bridge-status");
const detail = get<HTMLElement>("#bridge-detail");
const checkButton = get<HTMLButtonElement>("#check-bridge");
const saveStatus = get<HTMLElement>("#save-status");
const recentStatus = get<HTMLElement>("#recent-status");
const recentList = get<HTMLUListElement>("#recent-drops");
const preview = get<HTMLElement>("#drop-preview");
const textInput = get<HTMLTextAreaElement>("#drop-text");
const textButton = get<HTMLButtonElement>("#save-text");
const pickButton = get<HTMLButtonElement>("#choose-image");
const cameraButton = get<HTMLButtonElement>("#take-photo");
const imagePicker = get<HTMLInputElement>("#image-picker");
const cameraPicker = get<HTMLInputElement>("#camera-picker");
const deleteDialog = get<HTMLDialogElement>("#delete-dialog");
const deleteItem = get<HTMLElement>("#delete-item");
const deleteStatus = get<HTMLElement>("#delete-status");
const cancelDelete = get<HTMLButtonElement>("#cancel-delete");
const confirmDelete = get<HTMLButtonElement>("#confirm-delete");
const modelStatusView = get<HTMLElement>("#model-status");
const retryModelButton = get<HTMLButtonElement>("#retry-model-download");
const allowMobileModelButton = get<HTMLButtonElement>("#allow-mobile-download");
const pauseModelButton = get<HTMLButtonElement>("#pause-model-download");
const modelProgress = get<HTMLProgressElement>("#model-progress");
const modelDownloadPercent = get<HTMLElement>("#model-download-percent");
const goalDialog = get<HTMLDialogElement>("#goal-dialog");
const goalConfirmSummary = get<HTMLElement>("#goal-confirm-summary");
const goalConfirmStatus = get<HTMLElement>("#goal-confirm-status");
const goalCancel = get<HTMLButtonElement>("#goal-cancel");
const goalCommit = get<HTMLButtonElement>("#goal-commit");
const parameterDialog = get<HTMLDialogElement>("#parameter-dialog");
const parameterConfirmSummary = get<HTMLElement>("#parameter-confirm-summary");
const parameterConfirmStatus = get<HTMLElement>("#parameter-confirm-status");
const parameterCancel = get<HTMLButtonElement>("#parameter-cancel");
const parameterCommit = get<HTMLButtonElement>("#parameter-commit");

const rust: InvokeAdapter = (command, args) => invoke(command, args);
let saving = false;
let selectedAssetId: string | null = null;
let deleteTarget: import("./platform/local-drops").DropSummary | null = null;
let deleteReturnFocus: HTMLElement | null = null;
let goalConfirmAction: (() => Promise<void>) | null = null;
let goalConfirmBusy = false;
let goalReturnFocus: HTMLElement | null = null;
let parameterConfirmAction: (() => Promise<void>) | null = null;
let parameterConfirmBusy = false;
let parameterReturnFocus: HTMLElement | null = null;

const errors: Record<string, string> = {
  EMPTY_TEXT: "請先輸入文字；空白不會被存成收藏。",
  TEXT_TOO_LARGE: "這段文字超過暫定 64 KB 上限，請先分段保存。",
  EMPTY_IMAGE: "沒有收到圖片內容。",
  IMAGE_TOO_LARGE: "圖片超過暫定 8 MB 上限，請選較小的圖片。",
  CORRUPT_IMAGE: "圖片檔案已損毀或無法讀取。",
  UNSUPPORTED_OR_CORRUPT_IMAGE: "圖片格式不支援或檔案損毀，目前僅支援 PNG／JPEG／WebP。",
  IMAGE_TYPE_MISMATCH: "圖片內容與檔案格式不符，無法安全保存。",
  STORAGE_UNAVAILABLE: "手機私人儲存空間無法使用。",
  STORAGE_WRITE_FAILED: "手機寫入失敗，請檢查可用空間。",
  DATABASE_UNAVAILABLE: "私人資料庫無法開啟。",
  DATABASE_BUSY: "私人資料庫忙碌，請稍後再試。",
  DATABASE_WRITE_FAILED: "收藏未完成，請再試一次。",
  STORAGE_READ_FAILED: "內容讀取失敗，請稍後再試。",
  ASSET_FILE_MISSING: "找不到已保存的圖片。資料庫紀錄未被刪除，請回報這個錯誤。",
  ASSET_NOT_FOUND: "找不到這筆私人收藏，可能已被刪除。請重新整理。",
  INVALID_ASSET_ID: "收藏識別碼無效，操作已取消。",
  STORAGE_DELETE_FAILED: "手機無法刪除圖片；原收藏已保留，請稍後重試。",
  STORAGE_CLEANUP_PENDING: "收藏紀錄已移除，但私人圖片清理尚未完成；請關閉並重新開啟 App 重試清理。",
  STORAGE_RECOVERY_FAILED: "私人收藏檔案需要修復；尚未確認刪除成功，請重新開啟 App。",
  STORAGE_RECOVERY_REQUIRED: "刪除中途發生資料庫錯誤，請重新開啟 App 確認結果。",
  DATABASE_READ_FAILED: "私人資料庫讀取失敗，請稍後再試。",
  OCR_MODEL_UNAVAILABLE: "此裝置尚未提供 OCR 模型，請直接輸入人工修正。",
  OCR_RECOGNITION_FAILED: "手機本地 OCR 辨識失敗，請重試或人工輸入。",
  OCR_TIMEOUT: "手機 OCR 已超過 20 秒，請重試或人工修正；沒有轉送雲端。",
  OCR_INVALID_INPUT: "圖片辨識輸入無效，請重新選擇圖片。",
  OCR_INVALID_RESULT: "本地 OCR 回傳資料不合法，沒有採用辨識結果。",
  OCR_IMAGE_REQUIRED: "這不是圖片收藏，不需 OCR。",
  OCR_IMAGE_TOO_LARGE: "圖片像素過大，手機無法安全辨識；可人工輸入文字。",
  OCR_IMAGE_INVALID: "無法解析圖片內容，請直接人工輸入。",
  OCR_OUT_OF_MEMORY: "手機記憶體不足，已停止辨識；可人工輸入文字。",
  MODEL_UNAVAILABLE: "這台裝置尚未具備 Gemma 推論能力；不會切換到雲端 AI。",
  MODEL_NOT_INSTALLED: "尚未匯入經驗證的 Gemma 4 E2B 模型。",
  MODEL_IMPORT_CANCELLED: "已取消匯入，沒有更動私人資料。",
  MODEL_INVALID_FILE: "請選擇官方 gemma-4-E2B-it.litertlm 模型檔。",
  MODEL_HASH_MISMATCH: "模型完整性校驗不符，沒有安裝；請重新從官方來源下載。",
  MODEL_NO_SPACE: "手機儲存空間不足，模型約需 2.59 GB 加上暫存空間。",
  MODEL_IMPORT_FAILED: "匯入模型失敗，請檢查檔案與手機剩餘空間。",
  MODEL_DOWNLOAD_FAILED: "Gemma 模型下載中斷。已保存下載進度，請檢查網路後按「繼續下載」。",
  MODEL_DOWNLOAD_NETWORK: "網路暫時中斷，下載進度保留，重新連線後可續傳。",
  MODEL_LOAD_FAILED: "手機無法載入本地 Gemma 模型。",
  MODEL_INFERENCE_FAILED: "本地 Gemma 推論失敗，可稍後再試或自行修正。",
  MODEL_TIMEOUT: "Gemma 本地推論逾時，沒有使用雲端 AI。",
  MODEL_OUT_OF_MEMORY: "手機記憶體不足，已停止本地推論。",
  MODEL_BUSY: "本地模型正在處理其他工作，請稍後重試。",
  MODEL_INPUT_UNKNOWN: "vision_unknown：尚無可讀文字。請先 OCR、人工更正或只收藏；沒有圖片像素理解。",
  MODEL_INPUT_TOO_LARGE: "原始內容太長，暫時無法進行裝置端理解。",
  MODEL_INPUT_INVALID: "本地模型輸入不合法，操作已停止。",
  MODEL_INVALID_OUTPUT: "Gemma 輸出欄位不合法，系統沒有保存，也沒有建立任何 Goal。",
  MODEL_OUTPUT_FORMAT: "Gemma 回答格式不完整，已拒絕保存；可以再按一次理解。",
  MODEL_EVIDENCE_MISMATCH: "Gemma 引用的文字與原文不符，已拒絕保存；請檢查原文或重新執行。",
  MODEL_OUTPUT_TOO_LONG: "Gemma 回答超出安全長度，沒有保存；可以重新執行。",
  MODEL_OUTPUT_UNAPPROVED_FIELDS: "Gemma 回答涉及不能由 AI 決定的欄位，已拒絕保存。",
  MODEL_STALE_INPUT: "你已更新原文／OCR，舊的模型建議已失效，請重新推論。",
  GOAL_EMPTY: "請親自輸入你想找真人學伴做什麼；只有收藏字詞還不等於 Goal。",
  GOAL_TOO_LARGE: "目標敘述太長，請縮短至最多 500 字／2048 bytes。",
  GOAL_CONFIRMATION_REQUIRED: "未收到有效的本人明確確認操作，沒有建立草稿。",
  GOAL_REVISION_CONFLICT: "草稿已被更新，請重新開啟收藏查看最新版本後再確認。",
  INVALID_DRAFT_ID: "私人 Goal 或參數識別碼不合法，已拒絕操作。",
  INVALID_PARAMETER_INPUT: "條件名稱／內容無效或太長，請重新輸入。",
  INVALID_TYPE: "這個已知概念目前只允許明確核准的 N2 值，其他值不能假裝已確認。",
  INVALID_SCOPE: "目前只能管理 Intent 範圍的私人條件，Profile／Pair 尚未開放。",
  INVALID_EXPIRY: "一次性條件必須親自指定未來的完整日期與時間，不能猜測『週六』或改成每週。",
  INVALID_VERSION: "參數版本不合法，請重新開啟收藏。",
  PARAMETER_VERSION_CONFLICT: "條件已更新。請重新開啟收藏後，再修改或確認。",
  PARAMETER_CONFIRMATION_REQUIRED: "沒有收到本人明確確認，未改變參數狀態。",
  PARAMETER_UNAPPROVED_CONCEPT: "這是未知或未批准的私人條件，不能確認成正式配對概念。",
  PARAMETER_NOT_FOUND: "找不到條件或已從這份私人草稿移除。",
  B06_SLOT_BUDGET_EXCEEDED: "這個 Intent 的八個有效配對概念位置已滿。請先自行決定要保留哪些條件；系統不會默默刪除。",
  B06_SLOT_ADMISSION_REJECTED: "本地資料庫拒絕這個配對概念；請確認來源、本人確認及有效期限。",
  DATABASE_MIGRATION_FAILED: "本機參數資料庫初始化失敗，請不要重試公開分享。",

};

function errorMessage(error: unknown): string {
  const raw = error instanceof Error ? error.message : String(error);
  return errors[raw] ?? `操作未完成：${raw}`;
}

function setView(view: View): void {
  const diagnostic = view === "diagnostic";
  dropView.hidden = diagnostic;
  diagnosticView.hidden = !diagnostic;
  tabDrop.classList.toggle("active", !diagnostic);
  tabDiagnostic.classList.toggle("active", diagnostic);
  if (diagnostic) {
    tabDrop.removeAttribute("aria-current");
    tabDiagnostic.setAttribute("aria-current", "page");
  } else {
    tabDiagnostic.removeAttribute("aria-current");
    tabDrop.setAttribute("aria-current", "page");
  }
}

function updateBusy(isBusy: boolean): void {
  saving = isBusy;
  textButton.disabled = isBusy;
  pickButton.disabled = isBusy;
  cameraButton.disabled = isBusy;
  for (const button of recentList.querySelectorAll<HTMLButtonElement>(".delete-action")) {
    button.disabled = isBusy;
  }
}

function assertNative(): void {
  if (!isTauri()) throw new Error("此為網頁預覽，請使用 Android／iOS App 儲存私人資料。");
}


const displayDownloadMb = (bytes: number) => (bytes / 1_000_000).toFixed(0);
let lastAutoResumeTry = 0;
let statusUpdating = false;
function renderModelProvisioning(s: ModelStatus): void {
  const ready = s.installed && s.integrityVerified;
  const pending = s.downloadState === "downloading" || s.downloadState === "verifying";
  const pct = Math.floor(s.downloadedBytes / s.totalBytes * 100);
  modelProgress.hidden = !pending;
  modelProgress.value = pct;
  modelDownloadPercent.textContent = pending || s.downloadedBytes > 0 && !ready
    ? "已取得 " + displayDownloadMb(s.downloadedBytes) + " / " +
      displayDownloadMb(s.totalBytes) + " MB（" + pct + "%）"
    : "";
  retryModelButton.hidden = ready || pending;
  allowMobileModelButton.hidden = ready || pending;
  pauseModelButton.hidden = !pending;
  if (ready) {
    modelStatusView.textContent =
      "Gemma 4 E2B 已自動下載、通過 SHA-256 驗證並安裝，可以試本地推論。";
  } else if (s.downloadState === "waiting_wifi") {
    modelStatusView.textContent =
      "等待 Wi-Fi（2.59 GB）。連線後會自動續傳；若願意使用行動數據，可自行選擇。";
  } else if (s.downloadState === "downloading") {
    modelStatusView.textContent = "CAKAMI 正在自動準備本地 AI 模型，請保持 App 開啟…";
  } else if (s.downloadState === "verifying") {
    modelStatusView.textContent = "下載完成，手機正在進行 SHA-256 完整性驗證…";
  } else if (s.downloadState === "paused") {
    modelStatusView.textContent = "模型下載已暫停；已下載部分保留，隨時可以繼續。";
  } else if (s.downloadState === "error") {
    modelStatusView.textContent = errorMessage(s.downloadError || "MODEL_DOWNLOAD_FAILED");
  } else if (s.downloadState === "unavailable") {
    modelStatusView.textContent = "目前裝置未支援此模型（Android 版本須符合需求）。";
  } else {
    modelStatusView.textContent = "模型尚未準備好。CAKAMI 會在 Wi-Fi 可用時自動準備。";
  }
}

async function refreshModelStatus(): Promise<void> {
  if (!isTauri()) {
    modelStatusView.textContent = "Web 預覽無法下載或執行手機 AI 模型。";
    retryModelButton.hidden = true;
    allowMobileModelButton.hidden = true;
    pauseModelButton.hidden = true;
    return;
  }
  if (statusUpdating) return;
  statusUpdating = true;
  try {
    const result = await modelStatus(rust);
    renderModelProvisioning(result);
    // Wi-Fi may appear while the App stays open. Try again at most once/15s.
    if (result.downloadState === "waiting_wifi" && Date.now() - lastAutoResumeTry > 15_000) {
      lastAutoResumeTry = Date.now();
      renderModelProvisioning(await prepareModel(rust, false));
    }
  } catch (error) {
    modelStatusView.textContent = errorMessage(error);
  } finally {
    statusUpdating = false;
  }
}

/** One-stop first-launch setup: no SAF picker, no manual file or cloud inference. */
async function initializeModelProvisioning(): Promise<void> {
  if (!isTauri()) {
    await refreshModelStatus();
    return;
  }
  modelStatusView.textContent = "正在檢查手機狀態，若有 Wi-Fi 將自動下載模型…";
  try {
    renderModelProvisioning(await prepareModel(rust, false));
  } catch (error) {
    modelStatusView.textContent = errorMessage(error);
  }
  window.setInterval(() => { void refreshModelStatus(); }, 2500);
}

async function renderModelPanel(assetId: string, autoAnalyze = false): Promise<void> {
  if (selectedAssetId !== assetId) return;
  const panel = document.createElement("section");
  panel.className = "model-panel";
  panel.setAttribute("aria-label", "私人 Gemma 內容理解與意圖候選");
  const heading = document.createElement("h3");
  heading.textContent = "Gemma 私人理解與可能意圖";
  const state = document.createElement("p");
  state.className = "status";
  state.setAttribute("role", "status");
  state.setAttribute("aria-live", "polite");
  const output = document.createElement("div");
  output.className = "model-output";
  const button = document.createElement("button");
  button.type = "button";
  button.className = "secondary";
  button.textContent = "用手機 Gemma 理解這筆內容";
  const note = document.createElement("p");
  note.className = "fineprint";
  note.textContent = "只依保存的文字或 OCR 推論；不明確就不猜 Goal。任何建議都需本人修改、確認，且不代表分享同意。";
  panel.append(heading, state, output, button, note);
  preview.append(panel);

  function showSuggestion(result: ModelSuggestion): void {
    output.replaceChildren();
    const quote = document.createElement("p");
    quote.textContent = "原文證據：" + result.evidenceQuote;
    const hint = document.createElement("p");
    hint.textContent = "可能內容：" + (result.contentHint || "未知");
    const uncertain = document.createElement("p");
    uncertain.textContent = "仍不確定：" + (result.uncertainty || "尚未確認是否正確");
    output.append(quote, hint, uncertain);
    if (result.intentCandidate) {
      const intent = document.createElement("p");
      intent.textContent = "可能想做的事（" + result.intentCandidate.domain + "）：" +
        result.intentCandidate.goalText;
      const evidence = document.createElement("p");
      evidence.className = "fineprint";
      evidence.textContent = "提出此建議的原文：" + result.intentCandidate.evidenceQuote;
      const adopt = document.createElement("button");
      adopt.type = "button";
      adopt.className = "secondary";
      adopt.textContent = "採用這個建議，先修改再確認";
      adopt.addEventListener("click", () => {
        if (selectedAssetId !== assetId) return;
        const field = preview.querySelector<HTMLTextAreaElement>('textarea[data-goal-input="true"]');
        if (!field) {
          state.textContent = "Goal 編輯欄位尚未準備好，請再開啟這筆收藏。";
          return;
        }
        field.value = result.intentCandidate!.goalText;
        field.focus();
        state.textContent = "私人建議已填入 Goal 編輯欄。請核對或修改，再按「確認 Goal」；尚未保存或分享。";
      });
      output.append(intent, evidence, adopt);
    } else {
      const unknown = document.createElement("p");
      unknown.textContent = "目前找不到足夠證據推測你的合作目標；可以只收藏，或在下方自行說明。";
      output.append(unknown);
    }
    state.textContent = "Gemma 本地建議已保存（尚未本人確認） · " + result.elapsedMs + " ms";
  }
  try {
    const saved = await loadModelSuggestion(rust, assetId);
    if (selectedAssetId === assetId) {
      if (saved) showSuggestion(saved);
      else state.textContent = "尚無經來源核對的模型建議。";
    }
  } catch (error) {
    if (selectedAssetId === assetId) state.textContent = errorMessage(error);
  }
  button.addEventListener("click", () => {
    if (selectedAssetId !== assetId) return;
    button.disabled = true;
    state.textContent = "正在手機內執行 Gemma 4 E2B（不會連線到雲端 AI）…";
    void (async () => {
      try {
        const result = await proposeModel(rust, assetId);
        if (selectedAssetId === assetId) showSuggestion(result);
      } catch (error) {
        if (selectedAssetId === assetId) state.textContent = errorMessage(error);
      } finally {
        button.disabled = false;
      }
    })();
  });
  if (autoAnalyze) button.click();
}

async function refreshDrops(): Promise<void> {
  if (!isTauri()) {
    recentStatus.textContent = "目前在 Web 預覽，無法讀取手機私人 SQLite。";
    return;
  }
  recentStatus.textContent = "正在讀取手機私人收藏…";
  try {
    const items = await listDrops(rust);
    recentList.replaceChildren();
    recentStatus.textContent = items.length === 0
      ? "還沒有私人收藏。選擇圖片、拍照或貼上文字即可開始。"
      : `最近 ${items.length} 筆私人收藏（只存手機，不會上傳）`;

    for (const item of items) {
      const li = document.createElement("li");
      const button = document.createElement("button");
      button.type = "button";
      button.className = "asset-link";
      const title = document.createElement("span");
      title.className = "asset-title";
      title.textContent = item.kind === "image"
        ? `圖片：${item.fileName || "未命名"}`
        : `文字：${item.textExcerpt || "私人筆記"}`;
      const subtitle = document.createElement("span");
      subtitle.className = "asset-meta";
      subtitle.textContent = `${new Date(item.createdAtMs).toLocaleString("zh-TW")} · ${item.byteLength} bytes${item.isDuplicate ? " · 與先前內容重複" : ""}`;
      button.append(title, subtitle);
      button.addEventListener("click", () => { void openAsset(item.id); });
      const remove = document.createElement("button");
      remove.type = "button";
      remove.className = "delete-action";
      remove.textContent = "刪除";
      remove.setAttribute("aria-label", "刪除這筆私人收藏");
      remove.disabled = saving;
      remove.addEventListener("click", () => requestDelete(item));
      const row = document.createElement("div");
      row.className = "asset-row";
      row.append(button, remove);
      li.append(row);
      recentList.append(li);
    }
  } catch (error) {
    recentStatus.textContent = errorMessage(error);
  }
}

/** 刪除入口：確認視窗，不會直接呼叫 Rust；取消時沒有任何資料寫入。 */
function requestDelete(item: import("./platform/local-drops").DropSummary): void {
  if (saving || deleteDialog.open) return;
  deleteTarget = item;
  deleteReturnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  deleteItem.textContent = item.kind === "image"
    ? '圖片：' + (item.fileName || "未命名")
    : '文字：' + (item.textExcerpt || "私人筆記");
  deleteStatus.textContent = "";
  deleteDialog.showModal();
}

async function deleteSelected(): Promise<void> {
  const item = deleteTarget;
  if (!item || saving) return;
  updateBusy(true);
  confirmDelete.disabled = true;
  cancelDelete.disabled = true;
  deleteStatus.textContent = "正在從手機刪除私人收藏…";
  try {
    assertNative();
    await deleteDrop(rust, item.id);
    if (selectedAssetId === item.id) {
      selectedAssetId = null;
      preview.replaceChildren();
      preview.hidden = true;
    }
    deleteDialog.close();
    saveStatus.textContent = "私人收藏已刪除；重新開啟 App 後不應再次出現。";
    await refreshDrops();
  } catch (error) {
    deleteStatus.textContent = errorMessage(error);
  } finally {
    updateBusy(false);
    confirmDelete.disabled = false;
    cancelDelete.disabled = false;
  }
}


/** B03：只讀本機 SQLite；圖片原始 OCR 與本人人工更正分離。 */
async function renderOcrEditor(assetId: string): Promise<void> {
  if (selectedAssetId !== assetId) return;
  const panel = document.createElement("section");
  panel.className = "ocr-panel";
  panel.setAttribute("aria-label", "日文 OCR 與人工修正");

  const heading = document.createElement("h3");
  heading.textContent = "裝置端日文 OCR";
  const state = document.createElement("p");
  state.className = "status";
  state.setAttribute("role", "status");
  state.setAttribute("aria-live", "polite");
  const recognized = document.createElement("pre");
  recognized.className = "ocr-raw";
  const run = document.createElement("button");
  run.type = "button";
  run.className = "secondary";
  run.textContent = "用手機辨識日文";

  const field = document.createElement("label");
  field.className = "field-label";
  field.textContent = "人工修正辨識文字（私人）";
  const correction = document.createElement("textarea");
  correction.rows = 4;
  correction.placeholder = "辨識錯誤或無結果時，可自行輸入日文；這不是配對 Goal。";
  field.append(correction);
  const save = document.createElement("button");
  save.type = "button";
  save.className = "primary";
  save.textContent = "保存人工修正";
  const privacy = document.createElement("p");
  privacy.className = "fineprint";
  privacy.textContent = "圖片先經本地 ML Kit OCR，再用和純文字相同的 Gemma 意圖引擎；無文字保留 vision_unknown。Gemma 沒有接收圖片像素，也不使用雲端 AI。";
  panel.append(heading, state, recognized, run, field, save, privacy);
  preview.append(panel);

  function displayOcr(record: OcrRecord): void {
    state.textContent = record.status === "recognized"
      ? "已辨識日文（請核對）；" + (record.engineVersion || "") + " · " + (record.elapsedMs ?? "?") + " ms"
      : record.status === "unknown"
        ? "OCR 未辨識到可靠文字（UNKNOWN），可人工輸入。"
        : "尚未執行 OCR，可點擊辨識或直接人工輸入。";
    recognized.textContent = record.rawText || "（沒有 OCR 原始文字）";
    correction.value = record.correctedText ?? record.rawText ?? "";
  }

  try {
    displayOcr(await loadOcr(rust, assetId));
  } catch (error) {
    if (selectedAssetId === assetId) state.textContent = errorMessage(error);
  }
  if (selectedAssetId !== assetId) return;

  run.addEventListener("click", () => {
    if (selectedAssetId !== assetId) return;
    run.disabled = true;
    save.disabled = true;
    state.textContent = "手機本地 OCR 辨識中，不會把原圖送往雲端…";
    void (async () => {
      try {
        const result = await recognizeOcr(rust, assetId);
        if (selectedAssetId === assetId) {
          displayOcr(result);
          // US-01B: route OCR text through the existing Rust/Gemma command.
          preview.querySelector(".model-panel")?.remove();
          const readable = Boolean((result.correctedText ?? result.rawText ?? "").trim());
          void renderModelPanel(assetId, readable);
          if (!readable) {
            state.textContent = "vision_unknown：圖片沒有可讀文字，不能假裝看懂像素。可人工修正或只收藏。";
          }
        }
      } catch (error) {
        if (selectedAssetId === assetId) state.textContent = errorMessage(error);
      } finally {
        run.disabled = false;
        save.disabled = false;
      }
    })();
  });

  save.addEventListener("click", () => {
    if (selectedAssetId !== assetId) return;
    run.disabled = true;
    save.disabled = true;
    state.textContent = "正在保存人工修正到手機 SQLite…";
    void (async () => {
      try {
        const result = await correctOcr(rust, assetId, correction.value);
        if (selectedAssetId === assetId) {
          displayOcr(result);
          state.textContent = "人工修正已在手機保存；原始辨識結果仍保留，沒有建立 Goal。";
          // 來源變更後不可繼續顯示舊模型建議。
          preview.querySelector(".model-panel")?.remove();
          // Source SHA is rechecked by Rust: old intent candidates cannot be reused.
          void renderModelPanel(assetId, true);
        }
      } catch (error) {
        if (selectedAssetId === assetId) state.textContent = errorMessage(error);
      } finally {
        run.disabled = false;
        save.disabled = false;
      }
    })();
  });
}

/** B05 — Registry gate: only confirmed private Goals can own parameter proposals. */
async function renderParameterPanel(assetId: string, goal: PrivateGoalDraft): Promise<void> {
  if (selectedAssetId !== assetId) return;
  preview.querySelector(".parameter-panel")?.remove();
  const panel = document.createElement("section");
  panel.className = "parameter-panel";
  panel.setAttribute("aria-label", "私人配對條件與參數 Registry");
  const heading = document.createElement("h3");
  heading.textContent = "私人條件（B05 Registry／B06 最多 8 slots）";
  const guidance = document.createElement("p");
  guidance.className = "fineprint";
  guidance.textContent = "目前只有 N2／JLPT N2／日語二級是已知的同一種概念。其他輸入只會成為 UNKNOWN 私人候選，不會被當作正式配對條件。";
  const status = document.createElement("p");
  status.className = "status";
  status.setAttribute("role", "status");
  status.setAttribute("aria-live", "polite");

  const aliasField = document.createElement("label");
  aliasField.className = "field-label";
  aliasField.textContent = "想記錄的條件（概念或別名）";
  const aliasInput = document.createElement("input");
  aliasInput.type = "text";
  aliasInput.maxLength = 80;
  aliasInput.placeholder = "例如：JLPT N2";
  aliasField.append(aliasInput);

  const valueField = document.createElement("label");
  valueField.className = "field-label";
  valueField.textContent = "條件值（已輸入 N2 時可留空）";
  const valueInput = document.createElement("input");
  valueInput.type = "text";
  valueInput.maxLength = 200;
  valueInput.placeholder = "例如：N2";
  valueField.append(valueInput);

  const expiryField = document.createElement("label");
  expiryField.className = "field-label";
  expiryField.textContent = "這個條件有效多久？";
  const policy = document.createElement("select");
  const optionIntent = document.createElement("option");
  optionIntent.value = "intent_lifecycle";
  optionIntent.textContent = "只限這份私人 Goal 草稿";
  const optionFixed = document.createElement("option");
  optionFixed.value = "fixed";
  optionFixed.textContent = "一次性：指定日期及時間";
  policy.append(optionIntent, optionFixed);
  expiryField.append(policy);

  const dateField = document.createElement("label");
  dateField.className = "field-label";
  dateField.textContent = "一次性的日期／時間（不會自動每週重複）";
  const fixedDate = document.createElement("input");
  fixedDate.type = "datetime-local";
  dateField.append(fixedDate);
  dateField.hidden = true;
  policy.addEventListener("change", () => {
    dateField.hidden = policy.value !== "fixed";
  });

  const save = document.createElement("button");
  save.type = "button";
  save.className = "secondary";
  save.textContent = "保存私人條件候選";
  const list = document.createElement("ul");
  list.className = "parameter-list";
  panel.append(heading, guidance, status, aliasField, valueField, expiryField,
    dateField, save, list);
  preview.append(panel);

  let rows: PrivateParameter[] = [];
  const active = () => selectedAssetId === assetId && panel.isConnected;
  async function refresh(): Promise<void> {
    if (!active()) return;
    try {
      rows = await listParameters(rust, goal.id);
      if (!active()) return;
      list.replaceChildren();
      const used = rows.filter(item => item.matchSlotIndex !== null && !item.stale).length;
      status.textContent = used + "/8 個已確認的本地私人配對概念（不是八個必填問題）。" +
        " 仍無 Cloud 分享授權；T-01 完整白名單尚未核准。";
      if (!rows.length) {
        status.textContent += " 尚無任何私人條件；單純收藏／OCR／模型不會自動新增。";
      }
      for (const item of rows) {
        const li = document.createElement("li");
        const name = document.createElement("p");
        name.className = "parameter-label";
        name.textContent = item.conceptId
          ? item.conceptId + "：" + (item.valueJson || "UNKNOWN")
          : "UNKNOWN：" + item.requestedAlias;
        const summary = document.createElement("p");
        summary.className = "fineprint";
        const expiryText = item.expiryPolicy === "fixed" && item.expiresAtMs !== null
          ? "一次性有效至 " + new Date(item.expiresAtMs).toLocaleString("zh-TW")
          : "只限此私人草稿";
        summary.textContent = "來源：本人輸入 · 狀態：" + item.state +
          " · v" + item.version + " · " + expiryText +
          (item.stale ? " · Goal 已修改，此候選需重新保存" : "") +
          (item.matchSlotIndex === null ? "" :
            " · B06 私人 Slot " + (item.matchSlotIndex + 1) + "/8（未對外分享）");
        li.append(name, summary);
        if (item.state === "OBSERVED" && item.conceptId && !item.stale) {
          const button = document.createElement("button");
          button.type = "button";
          button.className = "secondary";
          button.textContent = "由本人確認這個私人條件";
          button.addEventListener("click", () => {
            if (!active() || parameterDialog.open || parameterConfirmBusy) return;
            parameterConfirmSummary.textContent =
              name.textContent + " · " + expiryText + " · 仍為私人內容";
            parameterConfirmStatus.textContent = "尚未確認；可返回修改。";
            parameterReturnFocus = document.activeElement instanceof HTMLElement
              ? document.activeElement : null;
            parameterConfirmAction = async () => {
              if (!active()) throw new Error("PARAMETER_NOT_FOUND");
              await confirmParameter(rust, goal.id, item.id, item.version);
              await refresh();
            };
            parameterDialog.showModal();
          });
          li.append(button);
        } else if (item.state === "UNKNOWN") {
          const blocked = document.createElement("p");
          blocked.className = "fineprint";
          blocked.textContent = "概念或值尚未核准，不能被確認或拿去找人。";
          li.append(blocked);
        }
        list.append(li);
      }
    } catch (error) {
      if (active()) status.textContent = errorMessage(error);
    }
  }

  save.addEventListener("click", () => {
    if (!active() || parameterConfirmBusy) return;
    const alias = aliasInput.value.trim();
    const value = valueInput.value.trim() || null;
    const expiryPolicy = policy.value === "fixed" ? "fixed" : "intent_lifecycle";
    const dateMs = expiryPolicy === "fixed" ? new Date(fixedDate.value).getTime() : null;
    if (expiryPolicy === "fixed" && (!fixedDate.value || !Number.isFinite(dateMs))) {
      status.textContent = errorMessage("INVALID_EXPIRY");
      return;
    }
    save.disabled = true;
    status.textContent = "正在由 Rust 驗證概念、期限與私人來源…";
    void (async () => {
      try {
        const previewResult = await previewConcept(rust, alias, value);
        const prior = rows.find(item => previewResult.conceptId
          ? item.conceptId === previewResult.conceptId
          : item.conceptId === null && item.requestedAlias === alias);
        const saved = await saveParameter(rust, {
          draftId: goal.id,
          expectedDraftRevision: goal.revision,
          conceptAlias: alias,
          value,
          scope: "intent",
          expiryPolicy,
          expiresAtMs: dateMs,
          expectedVersion: prior?.version ?? 0,
        });
        if (!active()) return;
        await refresh();
        if (active()) {
          status.textContent = saved.state === "UNKNOWN"
            ? "已保存 UNKNOWN 私人候選。未經 Registry 批准，不能確認或分享。"
            : "已保存私人條件（OBSERVED）。要正式確認這個條件，請另按本人確認。";
        }
      } catch (error) {
        if (active()) status.textContent = errorMessage(error);
      } finally {
        save.disabled = false;
      }
    })();
  });

  try {
    const registry = await listRegistry(rust);
    if (active() && registry.length === 0) {
      status.textContent = "尚無已批准的 Registry；未知輸入都只能保存為私人候選。";
    }
  } catch (error) {
    if (active()) status.textContent = errorMessage(error);
  }
  await refresh();
}

/** US-01A/B04：模型建議只能經本人點選採用，不能自行確認 Goal。 */
async function renderGoalPanel(assetId: string): Promise<void> {
  if (selectedAssetId !== assetId) return;
  const panel = document.createElement("section");
  panel.className = "goal-panel";
  panel.setAttribute("aria-label", "S02 私人內容與本人 Goal 分離");
  const heading = document.createElement("h3");
  heading.textContent = "你想和別人一起完成什麼？";
  const explanation = document.createElement("p");
  explanation.className = "fineprint";
  explanation.textContent = "Gemma 可以提出私人 Goal 候選，但「パクる」等無上下文內容不能直接當成找人意圖。你可以採用建議、修改，或只收藏。";
  const state = document.createElement("p");
  state.className = "status";
  state.setAttribute("role", "status");
  state.setAttribute("aria-live", "polite");
  const field = document.createElement("label");
  field.className = "field-label";
  field.textContent = "本人要確認的 Goal（可選用上方 AI 建議）";
  const input = document.createElement("textarea");
  input.rows = 3;
  input.dataset.goalInput = "true";
  input.maxLength = 500;
  input.placeholder = "例如：我想找真人學伴一起練 JLPT N2 口說。";
  field.append(input);
  const choices = document.createElement("div");
  choices.className = "goal-actions";
  const keep = document.createElement("button");
  keep.type = "button";
  keep.className = "secondary";
  keep.textContent = "只收藏內容";
  const confirm = document.createElement("button");
  confirm.type = "button";
  confirm.className = "primary";
  confirm.textContent = "確認 Goal，建立私人草稿";
  choices.append(keep, confirm);
  const disclaimer = document.createElement("p");
  disclaimer.className = "fineprint";
  disclaimer.textContent = "確認只會產生私人 Goal 與本人事件；B05／B06 另外驗證並限制最多 8 種本地概念。B08 分享授權尚未開放，絕不會自動發布。";
  panel.append(heading, explanation, state, field, choices, disclaimer);
  preview.append(panel);

  let current: PrivateGoalDraft | null = null;

  function showGoal(saved: PrivateGoalDraft): void {
    current = saved;
    input.value = saved.goalText;
    state.textContent = "Goal 已由本人確認 · 私人草稿 v" + saved.revision +
      " · 沒有分享授權，也沒有建立可公開的 Intent。";
    confirm.textContent = "再次確認修改後的私人 Goal";
    if (selectedAssetId === assetId) void renderParameterPanel(assetId, saved);
  }

  try {
    const existing = await loadPrivateGoalDraft(rust, assetId);
    if (selectedAssetId !== assetId) return;
    if (existing) {
      showGoal(existing);
    } else {
      state.textContent = "尚未確認任何找學伴 Goal；目前只有私人內容。";
    }
  } catch (error) {
    if (selectedAssetId === assetId) state.textContent = errorMessage(error);
  }
  if (selectedAssetId !== assetId) return;

  // 成功路徑 A：只收藏內容，絕不調用 confirm_private_goal。
  keep.addEventListener("click", () => {
    if (selectedAssetId !== assetId || goalDialog.open) return;
    state.textContent = current
      ? "繼續只收藏。既有私人 Goal 草稿沒有改動，也不會公開。"
      : "已選擇只收藏內容。沒有建立 Goal、配對 Intent，也不會搜尋真人。";
  });

  // 成功路徑 B：只有本人的明確確認，Rust 才能產生事件與私人草稿。
  confirm.addEventListener("click", () => {
    if (selectedAssetId !== assetId || goalDialog.open || goalConfirmBusy) return;
    const goalText = input.value.trim();
    if (!goalText) {
      state.textContent = errorMessage("GOAL_EMPTY");
      input.focus();
      return;
    }
    if (new TextEncoder().encode(goalText).byteLength > 2048) {
      state.textContent = errorMessage("GOAL_TOO_LARGE");
      input.focus();
      return;
    }
    const expectedRevision = current?.revision ?? 0;
    goalConfirmSummary.textContent = goalText;
    goalConfirmStatus.textContent = "尚未確認；你可以返回修改或只收藏。";
    goalReturnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    goalConfirmAction = async () => {
      const saved = await confirmPrivateGoal(rust, assetId, goalText, expectedRevision);
      if (selectedAssetId === assetId) showGoal(saved);
    };
    goalDialog.showModal();
  });
}

async function openAsset(id: string, autoAnalyze = false): Promise<void> {
  selectedAssetId = id;
  preview.hidden = false;
  preview.replaceChildren();
  const heading = document.createElement("h3");
  heading.textContent = "私人收藏內容";
  const waiting = document.createElement("p");
  waiting.textContent = "正在從本機讀取…";
  preview.append(heading, waiting);
  try {
    const asset = await loadDrop(rust, id);
    if (selectedAssetId !== id) return;
    waiting.remove();
    if (asset.summary.kind === "image" && asset.dataBase64) {
      const img = document.createElement("img");
      img.className = "asset-image";
      img.alt = asset.summary.fileName || "私人圖片";
      img.src = `data:${asset.summary.mimeType};base64,${asset.dataBase64}`;
      preview.append(img);
      await renderOcrEditor(id);
    } else {
      const content = document.createElement("pre");
      content.className = "asset-text";
      content.textContent = asset.text || "";
      preview.append(content);
    }
    if (selectedAssetId === id) await renderModelPanel(id, autoAnalyze);
    if (selectedAssetId === id) await renderGoalPanel(id);
    const note = document.createElement("p");
    note.className = "fineprint";
    note.textContent = "私人內容與本人 Goal 分開保存；即使確認 Goal，也只有本機私人草稿，不會自動公開或找學伴。";
    preview.append(note);
  } catch (error) {
    if (selectedAssetId === id) waiting.textContent = errorMessage(error);
  }
}

async function savePrivateText(): Promise<void> {
  if (saving) return;
  updateBusy(true);
  saveStatus.textContent = "正在保存私人文字…";
  try {
    assertNative();
    const saved = await saveText(rust, textInput.value);
    textInput.value = "";
    saveStatus.textContent = saved.isDuplicate
      ? "私人文字已保存；系統偵測到相同內容曾收藏，兩筆都保留。"
      : "私人文字已保存到手機。";
    await refreshDrops();
    await openAsset(saved.id, true);
  } catch (error) {
    saveStatus.textContent = errorMessage(error);
  } finally {
    updateBusy(false);
  }
}

async function decodeImage(file: File): Promise<string> {
  if (file.size > MAX_IMAGE_BYTES) throw new Error("IMAGE_TOO_LARGE");
  if (file.size === 0) throw new Error("EMPTY_IMAGE");

  // 在送進 Rust 前先由 WebView 真正解碼；損毀的假 PNG/JPEG 不能被當成有效原圖。
  const url = URL.createObjectURL(file);
  try {
    await new Promise<void>((resolve, reject) => {
      const probe = new Image();
      probe.onload = () => resolve();
      probe.onerror = () => reject(new Error("CORRUPT_IMAGE"));
      probe.src = url;
    });
  } finally {
    URL.revokeObjectURL(url);
  }

  return await new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onerror = () => reject(new Error("CORRUPT_IMAGE"));
    reader.onload = () => {
      const dataUrl = reader.result;
      const comma = typeof dataUrl === "string" ? dataUrl.indexOf(",") : -1;
      if (comma < 0) reject(new Error("CORRUPT_IMAGE"));
      else resolve((dataUrl as string).slice(comma + 1));
    };
    reader.readAsDataURL(file);
  });
}

async function savePrivateImage(file: File, source: ImageSource): Promise<void> {
  if (saving) return;
  updateBusy(true);
  saveStatus.textContent = "正在檢查圖片與保存私人原圖…";
  try {
    assertNative();
    const b64 = await decodeImage(file);
    const saved = await saveImage(rust, file.name, file.type, b64, source);
    saveStatus.textContent = saved.isDuplicate
      ? "圖片已保存在手機，偵測到重複內容；原紀錄不會被覆蓋。"
      : "圖片已成功保存在手機私人空間。";
    await refreshDrops();
    await openAsset(saved.id);
  } catch (error) {
    saveStatus.textContent = errorMessage(error);
  } finally {
    updateBusy(false);
  }
}

async function runBridgeCheck(): Promise<void> {
  if (!isTauri()) {
    status.textContent = "未連接：目前在一般瀏覽器";
    detail.textContent = "Web 預覽無法替代 Android／iOS 真機 Rust Bridge 驗收。請使用 Tauri 啟動 App。";
    return;
  }
  checkButton.disabled = true;
  status.textContent = "正在呼叫 Rust…";
  detail.textContent = "這不是雲端請求。";
  try {
    const result = await checkBridge(rust);
    status.textContent = "Rust Bridge 已回應";
    const aiState: Record<typeof result.nativeAi, string> = {
      model_verified: "Gemma 4 E2B 已安裝／校驗通過（推論仍須實測）",
      downloading: "Gemma 正在下載或驗證",
      waiting_wifi: "等待 Wi-Fi 下載 Gemma",
      paused: "Gemma 下載已暫停",
      download_error: "Gemma 模型下載有問題",
      unavailable: "此裝置暫不支援 Gemma",
      not_installed: "Gemma 尚未安裝",
      status_error: "無法取得 Gemma 模型狀態（不代表未安裝）"
    };
    detail.textContent = `平台：${result.platform} · 版本：${result.appVersion} · Rust Bridge 不需網路：是 · 雲端 AI：禁用 · 本地 AI：${aiState[result.nativeAi]}`;
  } catch (error) {
    status.textContent = "Rust Bridge 檢查失敗";
    detail.textContent = errorMessage(error);
  } finally {
    checkButton.disabled = false;
  }
}

tabDrop.addEventListener("click", () => setView("drop"));
tabDiagnostic.addEventListener("click", () => setView("diagnostic"));
get("#go-diagnostic").addEventListener("click", () => setView("diagnostic"));
get("#back-home").addEventListener("click", () => setView("drop"));
checkButton.addEventListener("click", () => { void runBridgeCheck(); });
textButton.addEventListener("click", () => { void savePrivateText(); });
pickButton.addEventListener("click", () => imagePicker.click());
cameraButton.addEventListener("click", () => cameraPicker.click());
get("#reload-drops").addEventListener("click", () => { void refreshDrops(); });
retryModelButton.addEventListener("click", () => {
  if (retryModelButton.disabled) return;
  retryModelButton.disabled = true;
  void (async () => {
    try {
      renderModelProvisioning(await prepareModel(rust, false));
    } catch (error) {
      modelStatusView.textContent = errorMessage(error);
    } finally {
      retryModelButton.disabled = false;
    }
  })();
});
allowMobileModelButton.addEventListener("click", () => {
  if (allowMobileModelButton.disabled) return;
  allowMobileModelButton.disabled = true;
  void (async () => {
    try {
      renderModelProvisioning(await prepareModel(rust, true));
    } catch (error) {
      modelStatusView.textContent = errorMessage(error);
    } finally {
      allowMobileModelButton.disabled = false;
    }
  })();
});
pauseModelButton.addEventListener("click", () => {
  if (pauseModelButton.disabled) return;
  pauseModelButton.disabled = true;
  void (async () => {
    try {
      renderModelProvisioning(await pauseModelDownload(rust));
    } catch (error) {
      modelStatusView.textContent = errorMessage(error);
    } finally {
      pauseModelButton.disabled = false;
    }
  })();
});
parameterCancel.addEventListener("click", () => parameterDialog.close());
parameterCommit.addEventListener("click", () => {
  if (!parameterConfirmAction || parameterConfirmBusy) return;
  const action = parameterConfirmAction;
  parameterConfirmBusy = true;
  parameterCancel.disabled = true;
  parameterCommit.disabled = true;
  parameterConfirmStatus.textContent = "Rust 正在建立本人確認事件，仍不會分享或發布…";
  void (async () => {
    try {
      assertNative();
      await action();
      parameterDialog.close();
    } catch (error) {
      parameterConfirmStatus.textContent = errorMessage(error);
    } finally {
      parameterConfirmBusy = false;
      parameterCancel.disabled = false;
      parameterCommit.disabled = false;
    }
  })();
});
parameterDialog.addEventListener("cancel", event => {
  if (parameterConfirmBusy) event.preventDefault();
});
parameterDialog.addEventListener("close", () => {
  parameterConfirmAction = null;
  parameterReturnFocus?.focus();
  parameterReturnFocus = null;
});
goalCancel.addEventListener("click", () => goalDialog.close());
goalCommit.addEventListener("click", () => {
  if (!goalConfirmAction || goalConfirmBusy) return;
  const operation = goalConfirmAction;
  goalConfirmBusy = true;
  goalCommit.disabled = true;
  goalCancel.disabled = true;
  goalConfirmStatus.textContent = "正在由 Rust 建立本人確認事件與私人 Goal 草稿…";
  void (async () => {
    try {
      assertNative();
      await operation();
      goalDialog.close();
    } catch (error) {
      goalConfirmStatus.textContent = errorMessage(error);
    } finally {
      goalConfirmBusy = false;
      goalCommit.disabled = false;
      goalCancel.disabled = false;
    }
  })();
});
goalDialog.addEventListener("cancel", (event) => {
  if (goalConfirmBusy) event.preventDefault();
});
goalDialog.addEventListener("close", () => {
  goalConfirmAction = null;
  goalReturnFocus?.focus();
  goalReturnFocus = null;
});
cancelDelete.addEventListener("click", () => deleteDialog.close());
confirmDelete.addEventListener("click", () => { void deleteSelected(); });
deleteDialog.addEventListener("cancel", (event) => {
  if (saving) event.preventDefault();
});
deleteDialog.addEventListener("close", () => {
  deleteTarget = null;
  deleteReturnFocus?.focus();
  deleteReturnFocus = null;
});

imagePicker.addEventListener("change", () => {
  const file = imagePicker.files?.[0];
  if (file) void savePrivateImage(file, "picker");
  else saveStatus.textContent = "已取消選圖，沒有儲存新內容。";
  imagePicker.value = "";
});
cameraPicker.addEventListener("change", () => {
  const file = cameraPicker.files?.[0];
  if (file) void savePrivateImage(file, "camera");
  else saveStatus.textContent = "已取消拍照，沒有儲存新內容。";
  cameraPicker.value = "";
});

void refreshDrops();
void initializeModelProvisioning();
