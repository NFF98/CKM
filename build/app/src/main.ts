import { invoke, isTauri } from "@tauri-apps/api/core";
import { checkBridge, type InvokeAdapter } from "./platform/bridge";
import {
  MAX_IMAGE_BYTES,
  loadDrop,
  listDrops,
  saveImage,
  saveText,
  type DropSummary,
  type ImageSource,
} from "./platform/local-drops";
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
        <p class="eyebrow">SPRINT 1 · B02</p>
        <h1>把想學的日文，放進私人收藏。</h1>
        <p class="lead">選擇截圖、拍照或貼上文字。保存在手機裡，重啟後仍可找回；現在不會自動找人，也不會上傳原圖。</p>

        <section class="note" aria-labelledby="drop-heading">
          <h2 id="drop-heading">放入圖片或文字</h2>
          <div class="actions">
            <button type="button" class="primary" id="choose-image">選擇圖片</button>
            <button type="button" class="secondary" id="take-photo">拍照</button>
          </div>
          <input type="file" id="image-picker" accept="image/png,image/jpeg,image/webp" hidden aria-label="選擇私人圖片" />
          <input type="file" id="camera-picker" accept="image/png,image/jpeg,image/webp" capture="environment" hidden aria-label="手機拍照" />
          <label for="drop-text" class="field-label">或貼上日文文字</label>
          <textarea id="drop-text" rows="4" placeholder="例如：パクる 是什麼意思？" spellcheck="false"></textarea>
          <button type="button" class="primary" id="save-text">保存私人文字</button>
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

        <div class="note boundary">
          <strong>B03 本地 AI 尚未接入</strong>
          <p>目前僅保存你提供的原始內容。尚未執行 OCR、沒有產生 Goal、Need、Offer，也沒有發布配對 Intent。</p>
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
          <p id="bridge-detail" class="detail">本地 AI 尚未接入。</p>
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

const rust: InvokeAdapter = (command, args) => invoke(command, args);
let saving = false;

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
  ASSET_FILE_MISSING: "找不到已保存的圖片，請回報這個錯誤。",
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
}

function assertNative(): void {
  if (!isTauri()) throw new Error("此為網頁預覽，請使用 Android／iOS App 儲存私人資料。");
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
      li.append(button);
      recentList.append(li);
    }
  } catch (error) {
    recentStatus.textContent = errorMessage(error);
  }
}

async function openAsset(id: string): Promise<void> {
  preview.hidden = false;
  preview.replaceChildren();
  const heading = document.createElement("h3");
  heading.textContent = "私人收藏內容";
  const waiting = document.createElement("p");
  waiting.textContent = "正在從本機讀取…";
  preview.append(heading, waiting);
  try {
    const asset = await loadDrop(rust, id);
    waiting.remove();
    if (asset.summary.kind === "image" && asset.dataBase64) {
      const img = document.createElement("img");
      img.className = "asset-image";
      img.alt = asset.summary.fileName || "私人圖片";
      img.src = `data:${asset.summary.mimeType};base64,${asset.dataBase64}`;
      preview.append(img);
    } else {
      const content = document.createElement("pre");
      content.className = "asset-text";
      content.textContent = asset.text || "";
      preview.append(content);
    }
    const note = document.createElement("p");
    note.className = "fineprint";
    note.textContent = "這是你自己保存的內容，尚未產生找學伴 Goal，也沒有分享。";
    preview.append(note);
  } catch (error) {
    waiting.textContent = errorMessage(error);
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
    await openAsset(saved.id);
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
    detail.textContent = `平台：${result.platform} · 版本：${result.appVersion} · 不需網路：是 · 雲端 AI：禁用 · 本地 AI：尚未接入`;
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
