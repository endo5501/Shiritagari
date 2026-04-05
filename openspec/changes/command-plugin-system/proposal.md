## Why

現在のチャットインターフェースは全入力を LLM チャットとして処理するが、タイマーやヘルプ表示など LLM を経由しない即座のアクションを実行する手段がない。また、今後機能を追加する際に毎回 `send_message` を改修するのではなく、プラグインとして後付けできる拡張ポイントが必要。

## What Changes

- `send_message` に `/` プレフィックスによるコマンドルーティングを追加
- `CommandPlugin` trait による拡張可能なプラグインシステムを導入
- `CommandRouter` でプラグインの登録・ディスパッチを管理
- サンプル実装として `TimerPlugin`（`/timer <時間>`）を追加
- サンプル実装として `HelpPlugin`（`/help`）を追加

## Capabilities

### New Capabilities
- `command-routing`: スラッシュコマンドの解析・ルーティング・プラグイン登録の仕組み
- `timer-command`: `/timer <時間>` でワンショットタイマーを設定し、時間経過後に吹き出しで通知する機能

### Modified Capabilities
- `chat-interface`: `send_message` が `/` で始まるメッセージをコマンドとして処理し、マッチしなければ従来のチャット処理にフォールバックする

## Impact

- `src-tauri/src/lib.rs`: `send_message` にコマンドルーティングの分岐を追加、`AppState` に `CommandRouter` を保持
- `src-tauri/src/commands/` モジュールを新設（types.rs, router.rs, timer.rs, help.rs）
- フロントエンド変更なし（入力欄はそのまま、バックエンド側で分岐）
- 新規依存: なし（tokio::spawn + tokio::time::sleep で十分）
