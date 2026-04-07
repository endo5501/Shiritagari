## 1. Rust側: Config の Serialize 対応と保存機能

- [x] 1.1 `AppConfig` および全サブ構造体に `Serialize` derive を追加する
- [x] 1.2 `AppConfig::save(&self, path: &Path)` メソッドを実装する（既存ファイル読み込み→マージ→書き込み）
- [x] 1.3 config の save/load に対するユニットテストを追加する

## 2. Rust側: AppState.config を Arc<RwLock<AppConfig>> に変更

- [x] 2.1 `AppState.config` を `Arc<RwLock<AppConfig>>` に変更する
- [x] 2.2 既存の config 参照箇所（`get_mascot_config`, `send_message`, `answer_question`）を RwLock 対応に修正する
- [x] 2.3 ポーリングループの config 参照を `Arc<RwLock<AppConfig>>` 経由に変更する
- [x] 2.4 ポーリング間隔の動的変更のために `tokio::watch` チャネルを導入し、polling loop 側で `tokio::select!` により interval 変更を検知して ticker を再作成する

## 3. Rust側: 新規 Tauri コマンド

- [x] 3.1 `get_config` コマンドを実装する（現在の設定を JSON で返す）
- [x] 3.2 `save_config` コマンドを実装する（設定を受け取り、config.toml に保存し、AppState を更新する）
- [x] 3.3 `list_ollama_models` コマンドを実装する（Ollama API `/api/tags` からモデル一覧を取得して返す。Base URL をパラメータとして受け取る）
- [x] 3.4 各コマンドのユニットテストを追加する（Tauriコマンドは State 依存のため、核心ロジックは AppConfig::save テストで担保）

## 4. Rust側: トレイメニューと設定ウィンドウ

- [x] 4.1 トレイメニューに「Settings」項目を追加する
- [x] 4.2 「Settings」クリック時に `WebviewWindowBuilder` で設定ウィンドウ（約500x600）を動的に生成する。既存ウィンドウがあればフォーカスする
- [x] 4.3 新規 Tauri コマンドを `invoke_handler` に登録する

## 5. Frontend: 設定ウィンドウのエントリポイント

- [x] 5.1 `src/settings.tsx` エントリポイントと `settings.html` を作成する
- [x] 5.2 `vite.config.ts` にマルチページ設定を追加する

## 6. Frontend: Settings コンポーネント

- [x] 6.1 `SettingsApp` ルートコンポーネントを作成する（設定読み込み、保存、キャンセルのハンドリング）
- [x] 6.2 LLM設定セクションを実装する（プロバイダ選択ドロップダウン、プロバイダに応じた動的フォーム切り替え）
- [x] 6.3 Ollamaモデル選択を実装する（モデル一覧取得、ドロップダウン表示、更新ボタン、エラー時の手動入力フォールバック、Base URL変更時の再取得）
- [x] 6.4 監視設定セクションを実装する（ポーリング間隔入力、allowlist/blocklist のリスト編集UI）
- [x] 6.5 外観設定セクションを実装する（マスコット画像パス入力）
- [x] 6.6 設定画面のスタイリングを行う

## 7. Frontend: テスト

- [x] 7.1 SettingsApp コンポーネントの基本テスト（読み込み、保存、キャンセル）を追加する
- [x] 7.2 プロバイダ切り替えによる動的フォーム表示のテストを追加する
- [x] 7.3 Ollamaモデル一覧取得とフォールバックのテストを追加する

## 8. 最終確認

- [x] 8.1 `/simplify`スキルを使用してコードレビューを実施
- [x] 8.2 `/codex:review --scope branch --background` スキルを使用して現在開発中のコードレビューを実施
- [x] 8.3 `/opsx:verify`でchangeを検証
