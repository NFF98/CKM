import { invoke, isTauri } from "@tauri-apps/api/core";
import { checkBridge } from "./platform/bridge";
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
        <p class="eyebrow">SPRINT 1 · B01</p>
        <h1>把收藏的內容，變成真正的學習行動。</h1>
        <p class="lead">從日文單字、截圖或筆記開始。內容留在手機，找學伴前由你確認意圖與分享範圍。</p>
        <div class="note">
          <strong>目前已實作：App 骨架與 Rust Bridge</strong>
          <p>B02 圖片／文字 Drop、B03 本地 OCR／Gemma E2B 將在後續 Task 實作。現在不會自動傳送私人資料。</p>
        </div>
        <button type="button" class="primary" id="go-diagnostic">檢查裝置端 Rust 連線</button>
        <p class="fineprint">Phase 1 不使用雲端 AI。當前畫面不會呼叫伺服器。</p>
      </section>

      <section id="view-diagnostic" class="view" hidden>
        <p class="eyebrow">B01 · 連線診斷</p>
        <h1>裝置狀態</h1>
        <p class="lead">這項檢查呼叫手機 App 內的 Rust Command，不使用假的 API 回應。</p>
        <div class="diagnostic-panel">
          <p class="diagnostic-title">Rust Bridge</p>
          <p id="bridge-status" role="status" aria-live="polite">尚未執行檢查</p>
          <p id="bridge-detail" class="detail">完成真機測試前，不代表 B01 已通過驗收。</p>
        </div>
        <button type="button" class="primary" id="check-bridge">執行連線檢查</button>
        <button type="button" class="secondary" id="back-home">返回開始</button>
      </section>
    </main>
    <footer class="footer">CAKAMI · Local-first · 僅限 18 歲以上學習者</footer>
  </div>
`;

const dropView = document.querySelector<HTMLElement>("#view-drop")!;
const diagnosticView = document.querySelector<HTMLElement>("#view-diagnostic")!;
const tabDrop = document.querySelector<HTMLButtonElement>("#tab-drop")!;
const tabDiagnostic = document.querySelector<HTMLButtonElement>("#tab-diagnostic")!;
const status = document.querySelector<HTMLElement>("#bridge-status")!;
const detail = document.querySelector<HTMLElement>("#bridge-detail")!;
const checkButton = document.querySelector<HTMLButtonElement>("#check-bridge")!;

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
    const result = await checkBridge((command) => invoke(command));
    status.textContent = "Rust Bridge 已回應";
    detail.textContent = `平台：${result.platform} · 版本：${result.appVersion} · 不需網路：是 · 雲端 AI：禁用 · 本地 AI：尚未接入`;
  } catch (error) {
    status.textContent = "Rust Bridge 檢查失敗";
    detail.textContent = error instanceof Error ? error.message : String(error);
  } finally {
    checkButton.disabled = false;
  }
}

tabDrop.addEventListener("click", () => setView("drop"));
tabDiagnostic.addEventListener("click", () => setView("diagnostic"));
document.querySelector("#go-diagnostic")!.addEventListener("click", () => setView("diagnostic"));
document.querySelector("#back-home")!.addEventListener("click", () => setView("drop"));
checkButton.addEventListener("click", () => { void runBridgeCheck(); });
