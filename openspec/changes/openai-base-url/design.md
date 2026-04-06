## Context

現在のOpenAIプロバイダはエンドポイントが `https://api.openai.com/v1/chat/completions` にハードコードされている。一方、OllamaプロバイダはすでにOllamaと同様の `ollama_base_url` パラメータで任意のエンドポイントに接続可能。OpenAIプロバイダにも同じパターンを適用する。

## Goals / Non-Goals

**Goals:**
- `config.toml` の `openai_base_url` でOpenAIプロバイダの接続先を変更可能にする
- ローカルLLMサーバ（LM Studio、llama-server等）にAPIキーなしで接続可能にする
- 既存の設定との後方互換性を維持する

**Non-Goals:**
- Azure OpenAI固有の認証方式（`api-key`ヘッダー）への対応
- UIからのプロバイダ設定変更
- OpenAI互換APIのバリデーション（モデル一覧の取得など）

## Decisions

### Decision 1: `ollama_base_url` と同じパターンで `openai_base_url` を追加

OllamaプロバイダがすでにOllamaと同じアプローチを取っているため、同じパターンを踏襲する。

- `LlmConfig` に `openai_base_url: Option<String>` を追加
- `OpenAiProvider::new()` に `base_url: Option<String>` パラメータを追加
- 未指定時は現行の `https://api.openai.com` にフォールバック

**代替案**: プロバイダ共通の `base_url` フィールドにする → プロバイダごとにデフォルトURLが異なるため、個別フィールドの方がシンプル。

### Decision 2: APIキーをオプショナルにする

ローカルLLMサーバはAPIキーが不要なケースが多い。`openai_base_url` が設定されている場合、APIキーの環境変数が未設定でも空文字列でフォールバックし、起動を妨げないようにする。

- `openai_base_url` が設定されている → APIキーの環境変数が未設定なら空文字列を使用
- `openai_base_url` が未設定（＝本家OpenAI） → 現行通りAPIキー必須

**代替案**: 常にAPIキーをオプショナルにする → 本家OpenAI利用時にキー忘れのエラーが遅延する（API呼び出し時に401）ため、base_url有無で分岐する方が親切。

### Decision 3: `create_provider` のシグネチャを整理

現在 `ollama_base_url` は専用パラメータとして渡されている。`openai_base_url` も追加するとパラメータが増えすぎるため、ファクトリ関数に `LlmConfig` 全体を渡す形にリファクタリングする。

- `create_provider(provider_name, model, api_key_env, ollama_base_url)` → `create_provider(provider_name, model, api_key_env, config)` のように、base_url系の設定をまとめて渡す
- あるいはシンプルに `openai_base_url` パラメータを追加する

→ 今回はシンプルに `openai_base_url` パラメータを追加する方式を採用。パラメータが5つになるが、将来的にプロバイダが増えた際にリファクタリングする。

## Risks / Trade-offs

- [互換性] ローカルLLMサーバによってはOpenAI互換APIの実装が不完全な場合がある → ユーザ側で対処。本プロジェクトではOpenAI APIの標準レスポンス形式を前提とする。
- [APIキー空文字列] `Authorization: Bearer ` ヘッダーが空のBearerトークンで送信される → ローカルサーバでは通常無視されるため問題なし。
