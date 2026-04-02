## Why

PCで作業しているユーザの行動を能動的に観察・質問・学習する常駐型AIエージェントを構築する。既存のAIアシスタント（OpenClaw等）は「ユーザに頼まれたことをやる」受動的なモデルだが、Shiritagariは「自分から気になって聞きに行く」能動的なモデルを目指す。ユーザの操作を定期的に監視し、理解できない行動について質問し、その回答を記憶して将来の理解に活かすことで、使い込むほど賢くなる「育てる」体験を提供する。

## What Changes

- ActivityWatch REST APIと連携し、ユーザのウィンドウ操作を定期ポーリングで取得する機能を新規構築
- 3層ハイブリッド記憶モデル（パターン記憶・エピソード記憶・推測ログ）をSQLiteで実装
- LLMを用いた行動推論エンジン（操作ログ+記憶からユーザの行動を推測し、質問すべきか判定）
- confidence閾値ベースの質問判定ロジック（≥0.8で黙る、<0.8で質問、≤0.5で既存パターン再質問、≤0.3でソフトデリート）
- パターン記憶のconfidence時間経過減衰メカニズム
- エピソード記憶からパターン記憶への昇格ロジック（類似エピソード3回以上で昇格）
- Tauriベースの常駐チャットウィンドウUI（システムトレイ常駐）
- LLM Provider抽象化（Claude API / OpenAI API / Ollama等を設定で切替可能、推論用と対話用で分離可能）
- ユーザからの能動的なチャット対話機能（記憶を活用した応答）

## Capabilities

### New Capabilities
- `activity-observation`: ActivityWatch APIとの連携によるユーザ操作の定期ポーリングと取得
- `memory-store`: 3層ハイブリッド記憶モデル（パターン・エピソード・推測ログ）のSQLite実装と管理
- `behavior-inference`: LLMを用いた行動推論と質問判定（confidence閾値ベース）
- `llm-provider`: 複数LLMプロバイダの抽象化と設定ベースの切替
- `chat-interface`: Tauriベースの常駐チャットウィンドウUIとユーザ対話

### Modified Capabilities

(なし - 新規プロジェクト)

## Impact

- 新規Tauriプロジェクト（Rust + React + TypeScript）の作成
- ActivityWatch（localhost:5600）が稼働していることが前提
- LLM APIキーの設定が必要（ローカルLLM使用時は不要）
- SQLiteデータベースファイルのローカル保存
- macOS / Windows両対応のビルド設定
