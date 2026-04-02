## ADDED Requirements

### Requirement: 複数LLMプロバイダの切替
システムは設定ファイル(config.toml)で指定されたLLMプロバイダを使用しなければならない（SHALL）。初期対応プロバイダはClaude API、OpenAI API、Ollamaとする。

#### Scenario: Claude APIの使用
- **WHEN** config.tomlで `provider = "claude"` と設定されている時
- **THEN** Anthropic Claude APIを使用して推論・対話を行う

#### Scenario: OpenAI APIの使用
- **WHEN** config.tomlで `provider = "openai"` と設定されている時
- **THEN** OpenAI APIを使用して推論・対話を行う

#### Scenario: Ollamaの使用
- **WHEN** config.tomlで `provider = "ollama"` と設定されている時
- **THEN** ローカルのOllama APIを使用して推論・対話を行う

### Requirement: 推論用と対話用で別モデルを指定可能
システムは推論（ポーリング判定）用と対話（チャット）用で異なるLLMプロバイダ・モデルを設定可能にしなければならない（SHALL）。

#### Scenario: 推論と対話で異なるモデル
- **WHEN** config.tomlで `inference_provider` と `chat_provider` が別々に設定されている時
- **THEN** ポーリング推論時にはinference_providerを、チャット対話時にはchat_providerを使用する

#### Scenario: 単一モデル設定
- **WHEN** config.tomlで `provider` のみ設定され、個別設定がない時
- **THEN** 推論・対話の両方で同一プロバイダ・モデルを使用する

### Requirement: APIキーの安全な管理
システムはAPIキーを環境変数から読み取らなければならない（SHALL）。設定ファイルにAPIキーを直接記載してはならない。

#### Scenario: 環境変数からのAPIキー取得
- **WHEN** LLMプロバイダの初期化時
- **THEN** config.tomlの `api_key_env` で指定された環境変数名からAPIキーを読み取る

### Requirement: デフォルトはローカルLLMモード
システムは初回起動時にローカルLLMプロバイダ（Ollama）をデフォルトとしなければならない（SHALL）。外部API（Claude、OpenAI）の使用は明示的なオプトインを必要とする。

#### Scenario: 初回起動時のデフォルト
- **WHEN** config.tomlにprovider設定がない状態で起動した時
- **THEN** provider = "ollama" をデフォルトとして動作する

#### Scenario: 外部APIへの切替
- **WHEN** ユーザがconfig.tomlでproviderを "claude" または "openai" に変更した時
- **THEN** 外部APIへウィンドウタイトル等の操作データが送信される旨を初回利用時に通知し、確認を求める

### Requirement: プライバシー保護のためのデータ制御
システムはLLMへの送信前に、モニタリング対象のフィルタリングとデータのリダクションを行わなければならない（SHALL）。

#### Scenario: アプリの許可リスト/拒否リスト
- **WHEN** config.tomlで `blocklist_apps = ["Signal", "1Password"]` が設定されている時
- **THEN** blocklist対象アプリのウィンドウイベントはLLMに送信せず、推論対象から除外する

#### Scenario: 機密パターンのリダクション
- **WHEN** LLMに送信するウィンドウタイトルに設定されたリダクションパターン（メールアドレス、URLトークン等）が含まれる時
- **THEN** 該当部分をマスキング（例: `[REDACTED]`）してからLLMに送信する
