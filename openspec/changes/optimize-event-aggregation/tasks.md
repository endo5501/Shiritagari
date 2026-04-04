## 1. イベント取得の時間制限

- [x] 1.1 `poll_once()` で start パラメータを `max(カーソル, 現在 - 30分)` に制限する
- [x] 1.2 カーソルが30分以上前の場合にカーソルを進める処理を追加する

## 2. イベント集約

- [x] 2.1 `AggregatedEvent` 構造体を `providers/types.rs` に追加する（app, title, total_duration_seconds, last_active）
- [x] 2.2 `InferenceInput` の events フィールドを `Vec<AggregatedEvent>` に変更する
- [x] 2.3 `check_patterns_and_gather_context()` にイベント集約ロジックを実装する（(app, title) でグルーピング、duration合計、last_active追跡）
- [x] 2.4 集約後のイベントを last_active 降順でソートし上位30件に制限する

## 3. プロンプト更新

- [x] 3.1 Ollama プロバイダの `build_inference_prompt()` を app 別グルーピング表示に更新する
- [x] 3.2 Claude プロバイダのプロンプト構築を同様に更新する
- [x] 3.3 OpenAI プロバイダのプロンプト構築を同様に更新する

## 4. LLMログ追加

- [x] 4.1 `call_llm()` に集約後イベント数とプロンプト文字数のログを追加する
- [x] 4.2 `lib.rs` の LLM 呼び出し前後に開始/完了/エラーと所要時間のログを追加する
