## Context

現在のShiritagariは400x600のチャットウィンドウUIを持つ。推論エンジンは毎サイクル `InferenceOutput` を生成しているが、`should_ask: true` の時のみ `shiritagari-question` イベントとしてフロントエンドに送信される。`should_ask: false` の場合はDBに保存されるだけでユーザには不可視。

Tauri 2はウィンドウの `transparent`/`decorations` 設定、`data-tauri-drag-region` によるドラッグ、`set_always_on_top` による最前面制御をネイティブにサポートしている。

## Goals / Non-Goals

**Goals:**
- チャットUIを伺か風デスクトップマスコットUIに置換する
- 推論結果を思考吹き出しとしてユーザに常時可視化する
- 質問時に発話吹き出しに切り替え、ウィンドウを最前面に浮上させる
- 指定ディレクトリの透過PNGをキャラクターとして表示する
- キャラクター画像のドラッグでウィンドウを移動可能にする

**Non-Goals:**
- 表情差分やアニメーション（将来対応）
- 吹き出し位置の画面端連動（固定位置で開始）
- チャット履歴の表示（入力欄のみ）
- 複数キャラクターの同時表示

## Decisions

### 1. ウィンドウ構成: 透過単一ウィンドウ

吹き出し・キャラクター・入力欄を1つの透過ウィンドウ内にレイアウトする。

**代替案**: 吹き出しとキャラクターを別ウィンドウにする → ウィンドウ間の位置同期が複雑になるため不採用。

**設定変更**:
- `tauri.conf.json`: `transparent: true`, `decorations: false`, ウィンドウサイズを 300x500 程度に
- `index.html` / CSS: `body` と `html` の背景を透明に

### 2. イベント設計: `shiritagari-thought` イベントの新設

推論結果を毎サイクルフロントエンドに送るため、新しいTauriイベントを追加する。

```
shiritagari-thought: { inference: string, confidence: number }
shiritagari-question: string  (既存、変更なし)
```

**発火タイミング**:
- `PatternMatchResult::Silent` → 思考イベントなし（既知パターンなので変化がない）
- `PatternMatchResult::NeedLlm` → LLM結果の `inference` を `shiritagari-thought` で送信
- `PatternMatchResult::ReAsk` → `shiritagari-question` で送信（既存通り）

**代替案**: `shiritagari-question` に統合してペイロード型で区別する → イベントのセマンティクスが曖昧になるため不採用。

### 3. フロントエンド状態管理

React状態を以下のように再設計する:

```typescript
// 状態
thought: string | null       // 最新の思考テキスト
question: string | null      // 未回答の質問テキスト
input: string                // 入力欄のテキスト
isLoading: boolean           // 送信中フラグ

// 導出値（stateではない）
isAsking = question !== null  // 質問中かどうか
bubbleText = isAsking ? question : thought
bubbleClass = isAsking ? "speech" : "thought"
```

`bubbleMode` は独立stateとせず `question`/`thought` から導出する。stale closure問題を回避し、状態の不整合を防ぐ。

**状態遷移**:
```
question=null, thought=null ──[shiritagari-thought]──▶ thought="..."
thought="..." ──[shiritagari-thought]──▶ thought更新
thought="..." ──[shiritagari-question]──▶ question="..." (最前面浮上)
question="..." ──[回答送信]──▶ question=null (thoughtは維持)
```

### 4. 吹き出しの視覚的区別

思考バブルと発話バブルをCSSで明確に区別する:

- **思考バブル**: 丸い角、`○` で繋がる尻尾、半透明背景、控えめな色調
- **発話バブル**: 角丸四角、`╲` 型の尻尾、不透明背景、目立つ色調

CSSの `::after` 疑似要素で尻尾を表現する。

### 5. ドラッグ移動: startDragging API を利用

キャラクター画像を `<div>` でラップし、`onMouseDown` で `getCurrentWindow().startDragging()` を呼び出す。`<img>` はreplaced elementのため `data-tauri-drag-region` が直接動作せず、また透過ウィンドウでは同属性の挙動が不安定なため、明示的なAPI呼び出しを採用。

capabilities に `core:window:allow-start-dragging` 権限の追加が必要。

### 6. 最前面制御: Rust側からの `set_always_on_top`

質問イベント発火時にRust側で `set_always_on_top(true)` を呼ぶ。回答受信後の `answer_question` コマンド内で `set_always_on_top(false)` に戻す。

**代替案**: フロントエンドからTauri APIで制御する → Rust側で一元管理する方がイベント発火と連動しやすいため不採用。

### 7. キャラクターPNG読み込み

設定ファイル (`config.toml`) に `character_image` パスを追加する:

```toml
[mascot]
character_image = "/path/to/character.png"
```

- フロントエンドからは Tauri の `convertFileSrc` API でローカルファイルを表示
- 未設定の場合はデフォルトのプレースホルダー画像（バンドル済み）を表示
- 推奨サイズ: 200x300px（Retina用に400x600pxの画像を2x表示）

### 8. LLMプロンプト変更

3つのプロバイダー（Claude, OpenAI, Ollama）の `build_inference_prompt` で、`inference` フィールドの説明を変更する:

```
"inference": "ユーザが何をしているかの推測（デスクトップマスコットの吹き出しに表示します。短く、独り言風に、40文字以内で。例:「Reactのコンポーネントを書いてるな...」）"
```

プロンプトの共通部分は3ファイルに同一の変更を適用する（プロンプト生成の共通化は別課題）。

## Risks / Trade-offs

**[macOS/Windows間の透過挙動差異]** → macOSでは `transparent: true` がそのまま動くが、Windows では `webview_install_mode` やグラフィックドライバの影響を受ける場合がある。→ まずmacOSで動作確認し、Windows対応は後続で検証する。

**[ウィンドウサイズの固定]** → 吹き出しテキストの長さによりレイアウトが崩れる可能性がある。→ 吹き出しテキストはLLMプロンプトで40文字以内に制限し、CSS側でも `overflow` 制御する。

**[ドラッグとクリックの競合]** → `data-tauri-drag-region` はドラッグのみ処理し、クリックイベントは透過する。将来キャラクタークリックで表情変更等を追加する場合は再検討が必要。

**[入力欄の透過背景での可読性]** → 透過ウィンドウ上の入力欄は背景によって見えにくくなる。→ 入力欄には不透明の背景色を付与する。
