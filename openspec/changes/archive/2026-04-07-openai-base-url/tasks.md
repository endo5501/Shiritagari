## 1. Config

- [x] 1.1 `LlmConfig` に `openai_base_url: Option<String>` フィールドを追加する
- [x] 1.2 `LlmConfig::default()` で `openai_base_url: None` を設定する

## 2. OpenAiProvider

- [x] 2.1 `OpenAiProvider` 構造体に `base_url: String` フィールドを追加する
- [x] 2.2 `OpenAiProvider::new()` に `base_url: Option<String>` パラメータを追加し、未指定時は `https://api.openai.com` をデフォルトにする
- [x] 2.3 `call_api()` のURL構成をハードコードから `format!("{}/v1/chat/completions", self.base_url)` に変更する
- [x] 2.4 `api_key` パラメータを `String` のまま維持し、空文字列を許容する

## 3. Factory

- [x] 3.1 `create_provider()` のシグネチャをリファクタリングし、`&LlmConfig` を受け取る形に変更する
- [x] 3.2 `openai` プロバイダ生成時に `base_url` を `OpenAiProvider::new()` に渡す
- [x] 3.3 `openai_base_url` が設定されている場合、APIキー環境変数が未設定でも空文字列でフォールバックする
- [x] 3.4 `create_inference_provider()` と `create_chat_provider()` から `openai_base_url` を渡す

## 4. Tests

- [x] 4.1 `config.rs`: `openai_base_url` を含むTOMLの読み込みテストを追加する
- [x] 4.2 `factory.rs`: `openai_base_url` 付きでのプロバイダ生成テストを追加する
- [x] 4.3 `openai.rs`: カスタム `base_url` が正しく設定されるユニットテストを追加する
