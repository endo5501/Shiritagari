## Why

ポーリングサイクルが10分ごとに動作しているにも関わらず、質問が一切表示されないケースがある。現状、各判定ステップ（AW接続確認、AFK判定、イベント取得、プライバシーフィルタ、パターンマッチング、LLM推論、プロバイダー生成）の結果がどこにも出力されないため、どこで処理が止まっているのか開発者が把握できない。`log` クレートと `env_logger` は依存に含まれているが初期化されておらず、既存の `info!`/`warn!` も出力されていない。

## What Changes

- `env_logger` を初期化し、`RUST_LOG` 環境変数でログレベルを制御可能にする
- ポーリングサイクルの各判定ポイントに構造化されたログ出力を追加する:
  - サイクル開始/終了
  - ActivityWatch 接続状態
  - AFK 判定結果
  - 取得イベント数（フィルタ前後）
  - パターンマッチング結果（アプリ名、信頼度、アクション）
  - LLM プロバイダー生成の成功/失敗
  - LLM 推論結果（should_ask、confidence）
  - 質問送信の有無
- `npm run tauri dev` 実行時にターミナルでこれらのログが確認できるようにする

## Capabilities

### New Capabilities

- `polling-debug-log`: ポーリングサイクルの判定過程を Rust の `log` クレート経由でターミナルに出力する機能

### Modified Capabilities

（なし — 既存の動作には変更を加えない。ログ出力の追加のみ）

## Impact

- **対象コード**: `src-tauri/src/lib.rs`（メインループ）、`src-tauri/src/polling/poller.rs`（ポーリング処理）、`src-tauri/src/inference/engine.rs`（推論判定）
- **依存**: `env_logger`（既に Cargo.toml に含まれている）
- **ランタイム影響**: ログ出力のオーバーヘッドのみ。`RUST_LOG` 未設定時はログが出力されないため本番動作に影響なし
