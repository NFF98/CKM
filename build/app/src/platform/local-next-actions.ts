/**
 * Offline next-action suggestions, separate from a person's confirmed Intent.
 * These are transparent, curated options, NOT claims made by Gemma or the user.
 * No network, no hidden goal creation and no cloud fallback.
 */
export type LocalNextAction = Readonly<{
  id: string;
  title: string;
  description: string;
  kind: "explain" | "save" | "draft";
  content: string;
}>;

export function suggestLocalNextActions(source: string): LocalNextAction[] {
  const text = source.trim();
  if (!text) return [];

  if (/(?:\bjlpt\b|日本語能力試験|日本語能力測驗|日檢)/i.test(text)) {
    const level = /(?:\b|日檢\s*)N([1-5])\b/i.exec(text)?.[1];
    const label = level ? "N" + level : "級別未指定";
    return [
      {
        id: "jlpt-levels", title: "看 JLPT 級別",
        description: "N5～N1 的難度與下一步",
        kind: "explain",
        content: "JLPT 有 N5、N4、N3、N2、N1 五個級別，N5 最基礎，N1 最進階。" +
          "目前輸入的目標是「" + label + "」。請先選你要準備哪一級，再決定詞彙、讀解或聽力練習。",
      },
      {
        id: "jlpt-note", title: "建立 JLPT 複習卡",
        description: "真的存進手機收藏，下次可再開",
        kind: "save",
        content: "【JLPT 複習卡】\n目標級別：" + label +
          "\n下一步：選擇目標 N5～N1；列出需要加強的詞彙、文法、讀解和聽力。\n備註：此卡由我自己選擇儲存，不代表已確認找學伴。",
      },
      {
        id: "jlpt-partner", title: "找人一起練日文",
        description: "先填好私人目標，確認前不會找人",
        kind: "draft",
        content: "我想找人一起準備 JLPT，練習日文；目標級別由我再確認。",
      },
    ];
  }

  if (/(?:[ぁ-ゟ゠-ヿ一-龯]|日語|日文|単語|日本語)/u.test(text)) {
    return [
      {
        id: "jp-review", title: "做一張日文複習卡", description: "保存原文到手機，之後可複習",
        kind: "save", content: "【日文複習卡】\n原文：" + text.slice(0, 200) +
          "\n詞義／用法：待本人核對。\n練習：用這個詞自己造句。",
      },
      {
        id: "jp-understand", title: "核對日文意思", description: "保留原文，不猜你的目的",
        kind: "explain", content: "請先核對文字、詞義與上下文。單靠一個單字不足以推斷你是要翻譯、造句、練口說還是找學伴。",
      },
      {
        id: "jp-partner", title: "一起練習日語", description: "帶入目標，仍要由你確認",
        kind: "draft", content: "我想找人一起練習日語，練習內容由我自己決定。",
      },
    ];
  }

  const tag = text.slice(0, 80);
  return [
    {
      id: "generic-review", title: "整理這筆內容",
      description: "把內容保存成可再開啟的私人筆記",
      kind: "save", content: "【我的內容筆記】\n" + tag + "\n下一步：寫下我想保留的重點。",
    },
    {
      id: "generic-understand", title: "先看原文再判斷",
      description: "避免 AI 自己猜你的目的",
      kind: "explain", content: "目前能確定的只有你提供了這筆內容；不能直接推斷你想買、學、分享或找人。你可以只收藏，或自行寫下想做的事。",
    },
    {
      id: "generic-goal", title: "自己設定下一步",
      description: "填入私人目標，尚不會公開",
      kind: "draft", content: "我想針對「" + tag + "」完成：",
    },
  ];
}
