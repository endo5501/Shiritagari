## 1. コマンド基盤

- [x] 1.1 `src-tauri/src/commands/` モジュールを作成し、`mod.rs` で公開する
- [x] 1.2 `commands/types.rs` に `CommandPlugin` trait、`CommandContext`（AppHandle, Database, plugin_list）、`CommandResult`、`PluginInfo` を定義する
- [x] 1.3 `commands/router.rs` に `CommandRouter` を実装する（register, dispatch, parse, plugin_list）
- [x] 1.4 `lib.rs` の `AppState` に `Arc<CommandRouter>` を追加し、`run()` 内で初期化する
- [x] 1.5 `send_message` に `app_handle: tauri::AppHandle` 引数を追加し、コマンドルーティング分岐を実装する（`/` 判定 → dispatch → 未知コマンドはエラー返却、`/` なしは既存チャット処理）

## 2. HelpPlugin

- [x] 2.1 `commands/help.rs` に `HelpPlugin` を実装する（`CommandContext.plugin_list` から一覧を生成して返す）

## 3. TimerPlugin

- [x] 3.1 `commands/timer.rs` に日本語時間パーサーを実装する（`N時間`, `N分`, `N秒`, 数値のみ→分、空白除去、ゼロ値エラー）
- [x] 3.2 `TimerPlugin` を実装する（`ctx.app_handle.clone()` を move して `tokio::spawn`、完了時に `shiritagari-thought` を `{ inference, confidence }` 形式で emit）
- [x] 3.3 引数なし・パース失敗・ゼロ値のエラーハンドリングを実装する

## 4. フロントエンド

- [x] 4.1 `App.tsx` の `handleSend` の catch ブロックで、エラーメッセージを `setThought` に表示するよう変更する

## 5. テスト

- [x] 5.1 コマンドルーティングのユニットテストを追加する（パース、dispatch、未登録コマンドのエラー、`/` 単体）
- [x] 5.2 時間パーサーのユニットテストを追加する（各パターン、空白入り、ゼロ値、パース不能）
- [x] 5.3 `App.test.tsx` にコマンドエラー表示のテストを追加する
- [x] 5.4 `npm run test:all` で全テスト通過を確認する
