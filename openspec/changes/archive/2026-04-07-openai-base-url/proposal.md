## Why

OpenAIプロバイダのAPIエンドポイントが `https://api.openai.com/v1/chat/completions` にハードコードされており、OpenAI互換APIを提供するローカルLLMサーバ（LM Studio、llama-server、vLLMなど）やAzure OpenAIなどに接続できない。`openai_base_url` 設定を追加することで、OpenAI互換APIを持つあらゆるサービスに対応可能にする。

## What Changes

- `LlmConfig` に `openai_base_url: Option<String>` フィールドを追加
- `OpenAiProvider` のコンストラクタで `base_url` を受け取り、APIリクエスト先を動的に構成
- `create_provider` ファクトリ関数に `openai_base_url` パラメータを追加
- APIキーが不要なローカルLLMサーバに対応するため、`openai` プロバイダ選択時にAPIキーが未設定でも起動可能にする（空文字列をフォールバック）

## Capabilities

### New Capabilities

（なし）

### Modified Capabilities

- `llm-provider`: OpenAIプロバイダにカスタムエンドポイント（`openai_base_url`）のサポートを追加。APIキーをオプショナル化。

## Impact

- `src-tauri/src/config.rs` — `LlmConfig` 構造体にフィールド追加
- `src-tauri/src/providers/openai.rs` — `base_url` フィールド追加、`call_api` のURL構成変更
- `src-tauri/src/providers/factory.rs` — `create_provider` の引数追加、OpenAI生成時に `base_url` を渡す
- 既存の設定ファイルは影響なし（新フィールドはオプショナル、デフォルトは現行の `https://api.openai.com`）
