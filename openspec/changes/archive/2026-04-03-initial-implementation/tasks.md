## 1. プロジェクトセットアップ

- [x] 1.1 Tauri v2 + React + TypeScriptプロジェクトの初期化（create-tauri-app）
- [x] 1.2 Rust側の依存クレート追加（tokio, reqwest, rusqlite, serde, toml）
- [x] 1.3 config.tomlの設定構造体定義と読み込み実装
- [x] 1.4 SQLiteデータベースの初期化とマイグレーション（テーブル作成）

## 2. 記憶ストア (memory-store)

- [x] 2.1 patternsテーブルのCRUD実装（作成、検索、confidence更新、ソフトデリート/復元/完全削除）
- [x] 2.2 episodesテーブルのCRUD実装（作成、検索、タグ付き検索）
- [x] 2.3 speculationsテーブルのCRUD実装（作成、検索、期限切れ削除）
- [x] 2.4 user_profileテーブルのCRUD実装
- [x] 2.5 confidence時間経過減衰の算出ロジック実装
- [x] 2.6 エピソード記憶の1ヶ月自動削除、推測ログの期限切れ削除、ソフトデリート済みパターンの30日後完全削除（定期クリーンアップ）
- [x] 2.7 エピソードからパターンへの昇格ロジック実装（類似エピソード3件以上で昇格）

## 3. ActivityWatch連携 (activity-observation)

- [x] 3.1 ActivityWatch REST APIクライアント実装（buckets一覧、events取得、カーソルベースのフェッチ）
- [x] 3.2 ポーリングループ実装（tokioベース定期タスク、間隔設定可能、カーソル永続化）
- [x] 3.3b イベント冪等性の保証（UNIQUE制約、UPSERT、カーソル更新のトランザクション一括コミット）
- [x] 3.3 AFK判定ロジック実装（AFKならスキップ）
- [x] 3.4 ActivityWatch未接続時のグレースフル動作（接続チェック、自動再開）

## 4. LLMプロバイダ (llm-provider)

- [x] 4.1 LlmProvider trait定義（infer, chat）
- [x] 4.2 Claude APIプロバイダ実装
- [x] 4.3 OpenAI APIプロバイダ実装
- [x] 4.4 Ollamaプロバイダ実装
- [x] 4.5 プロバイダファクトリ実装（設定に基づくプロバイダ生成、推論用/対話用分離、デフォルトOllama）
- [x] 4.6 外部API初回利用時の確認通知実装
- [x] 4.7 アプリ許可リスト/拒否リスト実装（blocklist対象アプリのイベント除外）
- [x] 4.8 リダクションパイプライン実装（メールアドレス、URLトークン等のマスキング）

## 5. 行動推論エンジン (behavior-inference)

- [x] 5.1 推論プロンプト設計（イベントログ+記憶+プロファイルを入力とするプロンプトテンプレート）
- [x] 5.2 パターン記憶照合ロジック実装（マッチ判定、confidence更新）
- [x] 5.3 LLM推論呼び出しと結果パース実装（inference, confidence, should_ask, suggested_question）
- [x] 5.4 質問判定ロジック実装（統一ステートマシン: silent/ask/re-ask/soft-delete、1ポーリング最大1質問）
- [x] 5.5 ポーリング→推論→質問/記録の統合フロー実装

## 6. チャットインターフェース (chat-interface)

- [x] 6.1 システムトレイ常駐の実装（Tauri v2 tray API）
- [x] 6.2 チャットウィンドウUIの実装（React: ChatWindow, MessageBubble, InputBar）
- [x] 6.3 Tauri IPC実装（バックエンド→フロントエンド: 質問イベント送信）
- [x] 6.4 Tauri IPC実装（フロントエンド→バックエンド: メッセージ送信、回答送信）
- [x] 6.5 ユーザからの能動的チャット対話の実装（記憶コンテキスト付きLLM呼び出し）

## 7. 統合とテスト

- [x] 7.1 全コンポーネントの統合（ポーリング→推論→質問→記憶→学習ループの結合）
- [x] 7.2 macOSでのビルドとテスト
- [x] 7.3 Windowsでのビルドとテスト
