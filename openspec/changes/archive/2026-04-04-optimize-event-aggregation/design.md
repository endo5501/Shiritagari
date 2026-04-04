## Context

現在、`poll_once()` はカーソル以降の全イベントをページネーションで取得し（最大50ページ=5000件）、`check_patterns_and_gather_context()` で1イベント1行のフラットな `EventSummary` に変換してLLMプロンプトに投入している。初回起動やカーソルリセット時に全履歴が流れ込み、llama3の8Kコンテキストに対して65K+トークンのプロンプトが生成される。

## Goals / Non-Goals

**Goals:**
- イベント取得を直近30分に制限し、不必要な大量取得を防止する
- イベントを app > title の二段構造に集約し、LLMにとって読みやすいコンパクトなプロンプトを生成する
- LLM呼び出しの可観測性を向上させる（ログ追加）

**Non-Goals:**
- 古いイベントの遡及処理（必要ならActivityWatchに直接問い合わせる）
- プロンプトテンプレートの大幅な見直し（集約結果のフォーマット変更のみ）
- イベント集約ロジックのカスタマイズ（設定項目の追加は行わない）

## Decisions

### 1. 時間制限はAPI取得時に適用する

`poll_once()` で `start` パラメータを `max(カーソル, 現在 - 30分)` とする。カーソルが30分以上前の場合、間のイベントは取得せずカーソルだけ進める。

**代替案: 取得後にフィルタ** — 5000件取得してから捨てるのは無駄。API段階で絞る方がネットワーク・メモリ効率が良い。

### 2. 集約は engine.rs の check_patterns_and_gather_context() 内で行う

プライバシーフィルタ後、LLMコンテキスト構築前に集約する。集約ロジックは `EventSummary` の構造変更を伴う。

**新しい EventSummary 構造:**
```rust
struct AggregatedEvent {
    app: String,
    title: String,
    total_duration_seconds: f64,
    last_active: String,  // 直近のタイムスタンプ（ソート用）
}
```

集約後の `InferenceInput` には `AggregatedEvent` のVecを渡す。app別グルーピングはプロンプト構築時（プロバイダ側）で行う。

### 3. 上位30件の選択は直近アクティブ順

`last_active`（各 `(app, title)` グループ内で最も新しいタイムスタンプ）の降順でソートし、上位30件を選択する。これにより「最近触っていたもの」が優先される。

### 4. プロンプトのフォーマットはapp別グルーピング

プロバイダの `build_inference_prompt()` で、集約済みイベントをapp別にグルーピングして表示する：

```
## 直近の操作ログ
Code (合計485秒)
  - main.rs (290秒)
  - lib.rs (120秒)
  - Cargo.toml (75秒)
Firefox (合計180秒)
  - GitHub - PR #123 (95秒)
  - GitHub - Issues (85秒)
```

### 5. LLMログは engine.rs と lib.rs に追加

- `engine.rs` の `call_llm()`: 集約後イベント数、プロンプト文字数を debug ログ
- `lib.rs`: LLM呼び出し開始を info、完了/エラーを info/warn でログ

## Risks / Trade-offs

- **[30分制限で重要なイベントを見逃す]** → 通常運用ではポーリング間隔（10分）以内のイベントしか未処理にならないため、30分は十分なバッファ。初回起動時に古い履歴を処理しないのは意図的な設計判断。
- **[title完全一致の集約粒度]** → 同じアプリ内でtitleが微妙に異なるイベント（例: ブラウザのタブ切り替え）は別エントリになる。上位30件の制限で自然に間引かれるため、過度な問題にはならない。
- **[EventSummary の構造変更]** → 既存の3プロバイダ（Ollama, Claude, OpenAI）のプロンプト構築を全て更新する必要がある。
