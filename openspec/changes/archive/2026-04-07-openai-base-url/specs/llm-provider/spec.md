## MODIFIED Requirements

### Requirement: 複数LLMプロバイダの切替
システムは設定ファイル(config.toml)で指定されたLLMプロバイダを使用しなければならない（SHALL）。初期対応プロバイダはClaude API、OpenAI API、Ollamaとする。OpenAIプロバイダは `openai_base_url` の設定によりカスタムエンドポイントに接続可能でなければならない（SHALL）。

#### Scenario: OpenAI APIの使用
- **WHEN** config.tomlで `provider = "openai"` と設定されている時
- **THEN** OpenAI APIを使用して推論・対話を行う（デフォルトエンドポイント: `https://api.openai.com`）

#### Scenario: OpenAI互換APIの使用
- **WHEN** config.tomlで `provider = "openai"` かつ `openai_base_url = "http://localhost:1234"` と設定されている時
- **THEN** 指定されたbase URLの `/v1/chat/completions` エンドポイントに接続して推論・対話を行う

#### Scenario: Claude APIの使用
- **WHEN** config.tomlで `provider = "claude"` と設定されている時
- **THEN** Anthropic Claude APIを使用して推論・対話を行う

#### Scenario: Ollamaの使用
- **WHEN** config.tomlで `provider = "ollama"` と設定されている時
- **THEN** ローカルのOllama APIを使用して推論・対話を行う

### Requirement: APIキーの安全な管理
システムはAPIキーを環境変数から読み取らなければならない（SHALL）。設定ファイルにAPIキーを直接記載してはならない。ただし、`openai_base_url` が設定されている場合、APIキーの環境変数が未設定でも空文字列をフォールバックとして使用し、起動を妨げてはならない（SHALL）。

#### Scenario: 環境変数からのAPIキー取得
- **WHEN** LLMプロバイダの初期化時
- **THEN** config.tomlの `api_key_env` で指定された環境変数名からAPIキーを読み取る

#### Scenario: カスタムエンドポイントでAPIキー未設定
- **WHEN** `openai_base_url` が設定されており、APIキーの環境変数が未設定の時
- **THEN** APIキーを空文字列としてプロバイダを初期化し、正常に起動する
