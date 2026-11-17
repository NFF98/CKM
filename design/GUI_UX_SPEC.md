# CAKAMI｜GUI / UX 詳細規格（審核草案）

> **狀態：DRAFT／待 Human Review，未 LOCKED、未授權開始 Coding。**  
> 日期：2026-11-17；範圍：Android-first，S01–S09、S04a；對應 Bible：B01–B04、B07–B09、B12–B18、B19、B21。  
> 規格優先級：CKM build/BACKLOG.md + Business v1.2／System v1.7／Functional v1.8；本文件只補畫面呈現與互動細節。若文字改變產品行為，必須先決策並同步正式設計。  
> **引用 APPf2 而不繼承產品功能**：APPf2 SSOT https://github.com/NFF98/appf2-design/blob/main/SSOT.md；APPf2 GUI Design System https://github.com/NFF98/appf2-design/blob/main/working/detailed-design/UI-UX/DESIGN-SYSTEM.md；APPf2 Screen Inventory https://github.com/NFF98/appf2-design/blob/main/working/detailed-design/UI-UX/PHASE1-SCREEN-INVENTORY.md。  
> 主 UI 文案使用繁體中文；日文學習內容保留日文。尚未批准任何語言切換機制。

## 已核准的 Phase 1 邊界：不使用雲端 AI（2026-11-17）

- **使用者明確決策**：Phase 1 MVP 不使用任何雲端 AI 推理。OCR、E2B 圖片／文字理解、意圖候選與必要語意特徵產生，必須在使用者裝置本地執行。
- GUI 不可出現「切換雲端 AI」「使用雲端模型再試一次」等入口；本地模型失敗時如實顯示，提供人工修正／保存本地的操作，不得偷偷改用 Cloud AI。
- **雲端資料服務仍存在**：使用者明確同意後，雲端可用於登入、已授權配對資料、資料庫查詢／比對、邀請與聊天；它們不是雲端模型推理。
- 此條為 User 已定的產品原則；其餘 GUI 畫面細節仍為 DRAFT。

## 1. 規格歸屬與鎖定方法

- **已確定（SOURCE）**：使用者 Drop → 本地內容辨識 → 親自確認 Goal → 預覽授權 → 找真人 → 必要追問 → 雙方同意 → Chat／15 分鐘練習 → 雙方確認結果。最多 8 個同時有效 Matching Concepts；每次最多 3 題主動追問（含 Goal 確認）；Pair-only 最多 2 題；無候選有 S04a；原圖／OCR 不自動上傳。
- **本文件提案（PROPOSED）**：畫面版型、具體按鈕位置、提示語、返回策略、狀態呈現、元件樣式。可以用來審核與製作 Low-fi，但不可當成已批准的實作事實。
- **未定（DECISION REQUIRED）**：第 6 節 D-UX-*；在 Human 確認前，不讓 Coding Agent 自行決定。
- 遵循 APPf2 的分工：GUI 文件擁有 composition／visual hierarchy／responsive／presentation；功能行為由 CAKAMI Functional Contract 擁有；System v1.7 擁有資料、授權與安全。APPf2 畫面 S01–S06 是另一產品，**不映射 CAKAMI 的 S 編號**。
- **已由 User 決定（2026-11-17）**：MVP 採 **Code-first GUI**；不製作、不要求 High-fi PNG／視覺 Mockup 作為 Coding 或驗收 Gate。每頁只先確認基本操作與重要狀態 → 共用 Design System → 直接實作 Android 可操作畫面 → 真機 Cross-screen／Functional／Privacy 驗收。APPf2 的 High-fi 圖只供設計語言參考，不當作 CAKAMI 的交付物。

## 2. 從 APPf2 參考的視覺共用原則（PROPOSED FOR CKM）

| 項目 | APPf2 參考基準 | CAKAMI 應用建議／限制 |
|---|---|---|
| 視覺氣質 | Clean Creator Canvas + Playful Energy | 乾淨、友善、有活力；學習配對是主角，不能做成 App Builder 或搜尋首頁 |
| 配色平衡 | 約 70% Neutral／White、20% Teal、10% Yellow | 作為視覺方向，非機械式面積比例；品牌移植仍待核准 |
| 主色 | Teal #0F766E；Hover #12958A | 主要操作、Focus、有效選取 |
| 輔色 | Aqua #2DD4BF；Yellow #F4C84C | 小面積強調；黃色不是 Warning；進度不顯示假完成 |
| 底色／文字 | #FFFFFF／#F7FAF9；主字 #102124；次字 #586865 | 安靜、清楚的 Android 閱讀介面 |
| 邊框 | #DDE8E6；較強邊框 #C8D9D6 | 卡片與欄位分界，不靠大量陰影 |
| 字體 | Inter + Noto Sans TC／系統 CJK fallback | 日文需保有可讀字形，不依賴外部字體檔才能首次啟動 |
| 字級 | 標題 24–32；內文 16；輔助 14；caption 12 | 主 CTA 最低約 14px；不以極小字承載授權條件 |
| 間距／圓角 | 4px grid；Mobile edge 16–20px；Button 12px；Card 16px；Sheet 20px | 全畫面共用，不自行建立任意尺寸 |
| 觸控 | Touch target ≥44 CSS px | 按鈕／圖示操作含鍵盤、無障礙名稱；Android back 行為一致 |
| 動畫 | 120／180／240ms；reduced motion | 不為了動畫延遲真實工作；不 fake progress |
| 狀態 | DEFAULT、FOCUS、PRESSED、DISABLED、LOADING、ERROR、SELECTED | 每個輸入與 CTA 必須有可驗收的狀態 |
| 危險與安全 | Info／Success／Warning／Danger／Critical 分色，搭配文字與圖示 | 撤銷尚未同步不能用成功色假裝完成；失敗不得只靠紅框 |
| 長等待 | 真 checkpoint 才顯示百分比，否則階段文字 + 活動指示器 | OCR／E2B／配對／同步均誠實呈現；不用假百分比 |

**不得直接複製**：APPf2 的「建立 App／修改 App」、Creator Capsule、舊 Bottom Navigation、Version Compare 流程、原產品 High-fi PNG、APPf2 商業範圍。品牌命名與 CAKAMI logo 尚未在 CKM 鎖定。

## 3. Android GUI 共用規則（待審核的實作提案）

1. **行動優先**：單欄、主要任務置前；控制列尊重 system status bar、keyboard 與 bottom safe area；不可讓主要 CTA 被鍵盤遮蓋。
2. **一頁一個主要決定**：優先引導下一個合法動作；不是每頁都要固定 Bottom Navigation。進入授權、邀請、確認等聚焦任務時避免誤跳轉。
3. **狀態必須誠實**：區分「仍在本地」「待本人確認」「已授權但同步中」「雲端生效」「撤銷尚未同步」「過期」「真正完成」。
4. **可返回但不假保存**：畫面離開不能無聲捨棄資料；若後端結果不確定，要標示目前持久化狀態。
5. **焦點／無障礙**：錯誤有文字與可回復操作；Focus visible；Screen Reader 順序與畫面一致；關閉 Modal 後恢復焦點。
6. **私人資料預設不外露**：S02 OCR、S03 Intent 私有內容不是候選卡的預設顯示資料；S05 只顯示經 Server 授權的摘要。
7. **載入與空狀態**：沒有資料不能用假真人卡片補版；網路離線不能顯示「已搜尋成功」；AI 不確定時必須允許人工修正。
8. **安全路徑可見**：候選卡、邀請／Chat 有可找到的封鎖／檢舉入口；拒絕必須真正禁止後續行為。
9. **界面文案**：對一般使用者使用「選擇圖片」「我想找學伴」「公開哪些資訊」「邀請對方」「練習已完成？」；不要把 JSON、scope、slot_index 等技術字顯示在消費者畫面。
10. **所有跨畫面限制**：不能讓 GUI 直接執行 SQLite Parameter 寫入；UI 發出指令由 Rust 驗證；雲端再次檢查。

## 4. S01–S09 逐頁 GUI Contract（SOURCE + PROPOSED）

| 畫面 | 使用者要完成什麼（SOURCE） | 低保真資訊區塊（PROPOSED） | 主要操作與可見狀態 |
|---|---|---|---|
| **S01 Drop** | 拍照／選圖／輸入文字，本地保存 | 極簡 Header；Drop 主區；拍照／選圖／文字入口；近期私人內容小區 | 「選擇圖片」「拍照」「輸入文字」；空白、權限拒絕、儲存中、存取失敗、離線 |
| **S02 Understand** | 看內容辨識候選並修正；確認是否真的想找學伴 | 原圖局部預覽、OCR／內容、AI 候選、可編輯區、Goal 確認 | 「修正內容」「我想找學伴」「只收藏內容」；辨識中、未知、修正、模型失敗、Goal 未確認 |
| **S03 Intent & Consent** | 管理 Intent、確認配對概念、預覽分享資料 | Goal 摘要、最多 8 個有效概念、缺少條件提示、私人／公開欄位預覽、授權區 | 「編輯」「確認分享並找學伴」「撤回分享」；未授權、已授權、8-slot 滿額、過期、同步失敗 |
| **S04 Search** | 搜尋已授權的真人 Intent | 目前尋找條件摘要、搜尋狀態、候選數量、可取消搜尋入口 | 「查看候選」「返回修改」；搜尋中、離線、無候選→S04a、伺服器失敗 |
| **S04a No Candidate** | 無候選時單人練習／可選通知 | 無候選說明、繼續本地學習的入口、新候選通知選項 | 「先自己練習」「稍後再找」；通知應為 Opt-in，不暗示已找到真人 |
| **S05 Candidate** | 查看真人候選與配對理由、略過／封鎖 | 已授權候選摘要、共同 Goal、可行的時間／互補條件、關鍵未知與理由 | 「查看詳細」「邀請／繼續確認」「略過」「封鎖」「檢舉」；候選過期、被封鎖、無候選、重新排序 |
| **S06 Pair Question** | 只回答真正需要補的 Pair-only 條件 | 候選上下文、單題問題、可選答案／UNKNOWN、問題計數 | 「確認答案」「現在不回答」「返回候選」；最多 2 題 Pair-only，必要時停配 |
| **S07 Mutual Consent** | 發送邀請、接受／拒絕、查看邀請狀態 | 對象及已授權摘要、邀請狀態、雙方是否同意、退出／安全入口 | 「發送邀請」「接受」「拒絕」「取消」「封鎖／檢舉」；等待中、拒絕、逾時、雙方接受 |
| **S08 Chat & Session** | 雙方接受後文字 Chat、15 分鐘共練 | 對象／Session 狀態、訊息區、輸入區、練習計時與取消／安全入口 | 「送出」「開始練習」「結束／取消」；發送中、斷線、對方離開、頻率限制、滿 15 分鐘 |
| **S09 Completion** | 各自確認真正完成、取消或 No-show | 時長與練習摘要、雙方確認狀態、結果選項 | 「確認完成」「未完成」「回到 Intent」；A 已確認/B 未確認、未滿時間、取消、完成 |

> **畫面功能邊界不可偷改**：S02 不能把辨識結果自動變成已確認 Goal；S03 不能把未授權欄位預設公開；S05 不可顯示模型捏造的人；S08 在雙方接受前不可開放；S09 不得以單方點擊計為完成。

## 5. 簡單文字示意（非獨立設計交付物；實際 Android App 才是 GUI 驗收對象）

### S01 — Drop
~~~text
┌────────────────────────────────┐
│ CAKAMI                         │
│                                │
│ 今天想練習什麼？               │
│ ┌────────────────────────────┐ │
│ │ 拖入／選擇 N2 筆記或截圖   │ │
│ │ [選擇圖片] [拍照]          │ │
│ └────────────────────────────┘ │
│ 或輸入日文／筆記               │
│ [__________________________]   │
│ [儲存並理解]                   │
│                                │
│ 私人內容只保存在這部手機       │
└────────────────────────────────┘
~~~

### S02 — Understand
~~~text
┌────────────────────────────────┐
│ ←   我看到了這些內容           │
│ [來源圖片預覽]                 │
│ 日文辨識結果  [編輯]           │
│ [パクる ...]                   │
│ AI 候選：N2 單字／文法         │
│ 不確定的地方：[未知]           │
│                                │
│ 你希望接下來做什麼？           │
│ [只保存學習內容]               │
│ [我想找真人學伴]               │
└────────────────────────────────┘
~~~

### S03 — Intent & Consent
~~~text
┌────────────────────────────────┐
│ ←   找學伴前，確認你的需求     │
│ 目標：練習 JLPT N2 文法 [編輯] │
│ 有效配對條件：X / 8           │
│ [條件卡] [條件卡] ...          │
│ 有缺少的條件？ [回答／略過]    │
│                                │
│ 只有以下資訊會分享：           │
│ [已核准摘要欄位的清單]         │
│ [明確授權選項]                 │
│ [確認分享並找學伴]             │
│ [只留在手機]                   │
└────────────────────────────────┘
~~~

### S05 — Candidate
~~~text
┌────────────────────────────────┐
│ ←   可能一起練習的學伴         │
│ ┌────────────────────────────┐ │
│ │ 候選人：已授權的稱呼       │ │
│ │ 相同目標：N2 文法          │ │
│ │ 可以一起練：15 分鐘文字    │ │
│ │ 為何推薦：可驗證匹配理由   │ │
│ │ 未知：共同練習時間         │ │
│ │ [補充條件] [邀請]          │ │
│ │ [略過] [封鎖／檢舉]        │ │
│ └────────────────────────────┘ │
└────────────────────────────────┘
~~~

### S07–S09 — 同意、聊天、結果
~~~text
S07 邀請卡 → 兩邊都接受才開 Chat
        ↓
S08 文字訊息 + 15 分鐘練習狀態
        ↓
S09 A 確認 + B 確認 + 有效時長
        ↓
      COMPLETED
~~~

### S04／S04a／S06 版型原則
- S04 不用大面積假的百分比；提供真實搜尋狀態。
- S04a 說明「目前沒有符合條件的真人」，不得模擬候選卡。
- S06 每次聚焦一個對配對有用的問題，顯示「不確定／稍後再說」的合法處理；不能為了湊問題數而追問。

## 6. 開發前必須決定的 GUI 問題（Open Decisions）

| ID | 尚未鎖定 | 影響 |
|---|---|---|
| D-UX-01 | CAKAMI 是否直接採用 APPf2 Teal／Yellow 作為品牌配色？Logo 是否另行設計？ | 所有 Screen 視覺 |
| D-UX-02 | 正式導航採單一流程 + 返回、底部導覽，或混合？Intent 清單入口在哪裡？ | B01、B07、全畫面 |
| D-UX-03 | S01 Drop 的預設入口與多張圖片操作細節、相機權限被拒後的返回行為 | B02–B03 |
| D-UX-04 | S02 的 Goal 提問文案與「只收藏」後畫面去向 | B04／B09 |
| D-UX-05 | S03 第 9 個必要條件卡的替換／合併／另建 Intent 互動，以及是否保留編輯中的 Draft | B05–B08 |
| D-UX-06 | S04a 單人練習要提供什麼具體互動；「新候選通知」的授權與通知管道 | B14、可能影響 B21 |
| D-UX-07 | 候選卡可公開的使用者欄位（稱呼、學習程度、上線時段等） | B11–B14／隱私 |
| D-UX-08 | S08 計時區、背景／離開 App 時的顯示及重返狀態 | B17–B18 |
| D-UX-09 | S09 未完成／No-show 的顯示措辭與避免誤觸策略 | B18 |
| D-UX-10 | Android 360／393／412px 寬度下的**實際 App** 佈局與真機裝置清單（不做 High-fi） | B19／B21 |

## 7. GUI Review 與驗收 Gate

| Gate | 證據 | 通過要求 |
|---|---|---|
| GUI-G01 | S01–S09、S04a 在**可操作 App** 的轉場核對 | 無缺頁、無斷線、主 CTA 有清楚去向 |
| GUI-G02 | 共用 Component／Token 審核 | 字級、色彩、觸控、互動狀態一致 |
| GUI-G03 | 逐頁 Android 實機操作／畫面證據 | 空、載入、錯誤、離線、封鎖／過期／撤銷狀態有呈現；**不要求 High-fi** |
| GUI-G04 | Cross-screen / Functional / Privacy Audit | 無前後矛盾、無未授權揭露、S07–S09 Gate 正確 |
| GUI-G05 | Human Approval + Git commit reference | 審核後才改標 LOCKED；未批准不能聲稱已經 GUI Freeze |

## 8. 追溯與後續動作

- 功能詳細規格：design/FUNCTIONAL_SPEC.md。
- 資料／介面細節：design/TECHNICAL_SPEC.md。
- 驗收測試：test/ACCEPTANCE_SPEC.md。
- 正式工作清單：build/BACKLOG.md。
- **MVP 不需要 approved Wireframe PNG／High-fi 圖片**。上方文字框線為實作溝通輔助；真正的 GUI Done 以 Android 可操作畫面 + 功能／隱私／無障礙測試證據為準。
