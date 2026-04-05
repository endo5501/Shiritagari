## 1. コマンド基盤

- [ ] 1.1 `src-tauri/src/commands/` モジュールを作成し、`mod.rs` で公開する
- [ ] 1.2 `commands/types.rs` に `CommandPlugin` trait、`CommandContext`、`CommandResult` を定義する
- [ ] 1.3 `commands/router.rs` に `CommandRouter` を実装する（register, dispatch, parse）
- [ ] 1.4 `lib.rs` の `AppState` に `Arc<CommandRouter>` を追加し、`run()` 内で初期化する
- [ ] 1.5 `send_message` にコマンドルーティング分岐を追加する（`/` 判定 → dispatch → フォールバック）

## 2. HelpPlugin

- [ ] 2.1 `commands/help.rs` に `HelpPlugin` を実装する（登録済みコマンド一覧を返す）
- [ ] 2.2 `CommandRouter` に全プラグインの name/description/usage を取得するメソッドを追加する

## 3. TimerPlugin

- [ ] 3.1 `commands/timer.rs` に日本語時間パーサーを実装する（`N時間`, `N分`, `N秒`, 数値のみ）
- [ ] 3.2 `TimerPlugin` を実装する（tokio::spawn で遅延後に `shiritagari-thought` を emit）
- [ ] 3.3 引数なし・パース失敗時のエラーハンドリングを実装する

## 4. 結合・テスト

- [ ] 4.1 `run()` で TimerPlugin と HelpPlugin を CommandRouter に登録する
- [ ] 4.2 コマンドルーティングのユニットテストを追加する（パース、dispatch、未登録コマンド）
- [ ] 4.3 時間パーサーのユニットテストを追加する（各パターン）
- [ ] 4.4 `npm run test:all` で全テスト通過を確認する
