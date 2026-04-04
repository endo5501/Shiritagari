## Why

起動直後やカーソルリセット時にActivityWatchの全履歴（最大5000件）がそのまま1:1でLLMプロンプトに変換され、Ollama (llama3, 8Kコンテキスト) に65K+トークンが投入されてGPUが過負荷になり、推論が完了しない。また、LLM呼び出し中のログが一切なく、問題の診断ができない。

## What Changes

- イベント取得に時間制限を導入し、直近30分以内のイベントのみ取得する（古いイベントはカーソルを進めて切り捨て）
- 取得したイベントを `(app, title)` で集約し、app > title の二段構造にまとめる（duration合計）
- 集約後、直近アクティブ順で上位30件のtitleに絞り込む
- LLM呼び出し前後にデバッグログを追加する（プロンプトサイズ、開始/完了/エラー）

## Capabilities

### New Capabilities
- `event-aggregation`: イベントの構造的集約（app別グルーピング、duration合計、上位N件選択）

### Modified Capabilities
- `activity-observation`: イベント取得に時間上限（直近30分）を追加
- `polling-debug-log`: LLM呼び出しのデバッグログを追加

## Impact

- `src-tauri/src/polling/poller.rs`: 時間制限付きイベント取得
- `src-tauri/src/inference/engine.rs`: イベント集約ロジック、LLMログ追加
- `src-tauri/src/providers/types.rs`: EventSummary構造の変更（構造的集約対応）
- `src-tauri/src/providers/ollama.rs`, `claude.rs`, `openai.rs`: プロンプト構築の変更
