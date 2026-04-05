## Context

現在 `send_message` は全入力を LLM チャットとして処理する。タイマーのような即座のアクションを実行する手段がなく、機能追加のたびに `send_message` を直接改修する必要がある。LLM Provider で既に trait ベースのプラグインパターンが確立されており、同じ手法をコマンドシステムに適用する。

今後 `/news`（ニュース検索）、`/daily-report`（日報作成）などのコマンドを追加予定のため、拡張可能な基盤が必要。

## Goals / Non-Goals

**Goals:**
- `CommandPlugin` trait による拡張可能なコマンドシステム
- 新コマンド追加 = ファイル追加 + 登録の1行、で済む構造
- `send_message` 内での透過的なルーティング
- サンプルとして Timer / Help プラグインを実装

**Non-Goals:**
- 外部プラグインの動的ロード（WASM 等）
- 設定ファイルによるプラグインの有効/無効切り替え
- 繰り返しタイマー（ポモドーロ等）
- タイマーの永続化（アプリ再起動で消失して構わない）

## Decisions

### 1. コマンドルーティングはバックエンド（Rust）側で行う

**選択**: `send_message` 内で `/` プレフィックスを検出し、`CommandRouter` にディスパッチ。`/` で始まるメッセージは常にコマンドとして扱い、未知のコマンドはエラーメッセージを返す（LLM チャットにはフォールバックしない）。

**理由**: UI 実装に依存しないため、フロントエンドの刷新時に影響を受けない。`/xxx` が意図せず LLM に流れるリスクを防ぐ。

**代替案**: フロントエンドで `/` を判定し別の Tauri コマンドを呼ぶ方式。関心の分離はきれいだがフロント変更が必要になる。

### 2. CommandPlugin trait の設計

```rust
pub struct CommandContext {
    pub app_handle: AppHandle,         // emit 用（clone 済み、'static）
    pub db: Arc<Mutex<Database>>,      // DB アクセス
    pub plugin_list: Vec<PluginInfo>,  // 登録済みプラグイン一覧（HelpPlugin 用）
}

pub struct PluginInfo {
    pub name: String,
    pub description: String,
    pub usage: String,
}

pub struct CommandResult {
    pub response: String,  // send_message の戻り値としてフロントに返すテキスト
}

#[async_trait]
pub trait CommandPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn usage(&self) -> &str;
    async fn execute(&self, args: &str, ctx: &CommandContext) -> Result<CommandResult, String>;
}
```

**選択**: `CommandContext` に `AppHandle`（clone 済み）、`Database`、`plugin_list` を含める。`CommandResult` は `response: String` のみとし、`send_message` の `Result<String, String>` にそのまま乗せる。

**理由**:
- `AppHandle` を clone して渡すことで、`tokio::spawn` 内での `'static` 要件を満たせる（TimerPlugin は `ctx.app_handle.clone()` を move して spawn）
- `plugin_list` を `Vec<PluginInfo>` として渡すことで、`HelpPlugin` が `CommandRouter` 自体への参照を必要としない
- `CommandResult.response` は `send_message -> Result<String>` にそのまま乗るため、フロントとの既存契約を維持

### 3. タイマーは tokio::spawn で管理

**選択**: `/timer 3時間` → `ctx.app_handle.clone()` を move して `tokio::spawn(async move { sleep(duration).await; emit(...) })`。

**理由**: サンプル実装として最小限。永続化不要。

**所有権モデル**: `execute` 内で `app_handle` を clone し、spawn に move する。`execute` 自体は即座に `CommandResult`（「タイマーを設定したよ！」）を返す。

### 4. タイマー完了通知の payload

**選択**: 既存の `shiritagari-thought` イベントを使い、payload は既存の `{ inference, confidence }` 形式に合わせる。

```rust
app_handle.emit("shiritagari-thought", &serde_json::json!({
    "inference": "⏰ タイマーが完了したよ！",
    "confidence": 1.0,
}))
```

**理由**: フロントエンドの `ThoughtPayload` インターフェースと既存テストを壊さない。

### 5. 時間パーサーの設計

日本語の時間表現（`3時間`, `30分`, `1時間30分`, `90秒`）を `Duration` に変換する。正規表現で `(\d+)時間`, `(\d+)分`, `(\d+)秒` をそれぞれ抽出し合算する。

境界条件:
- 数値のみ（`30`）→ 分として解釈
- 空白入り（`1時間 30分`）→ 空白を除去してからパース
- ゼロ値（`0分`）→ エラー（「0より大きい時間を指定してください」）
- パース不能 → エラー + usage 表示

### 6. CommandRouter の AppState への配置

**選択**: `AppState` に `Arc<CommandRouter>` を保持。`run()` 内で各プラグインを登録してから AppState に渡す。

**理由**: `send_message` から `State<AppState>` 経由でアクセスできる。

### 7. send_message の変更と AppHandle の取得

**選択**: `send_message` の引数に `app_handle: tauri::AppHandle` を追加（Tauri コマンドは `AppHandle` を直接受け取れる）。`/` で始まるメッセージは `CommandRouter::dispatch` に渡し、結果の `CommandResult.response` を `Ok(response)` として返す。`/` で始まらなければ既存のチャット処理。

```rust
#[tauri::command]
async fn send_message(
    message: String,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,  // 追加
) -> Result<String, String> {
    if message.starts_with('/') {
        let ctx = CommandContext {
            app_handle: app_handle.clone(),
            db: state.db.clone(),
            plugin_list: state.command_router.plugin_list(),
        };
        return state.command_router.dispatch(&message, &ctx).await;
    }
    // 既存のチャット処理...
}
```

### 8. フロントエンドの最小変更

**選択**: `send_message` の `catch` ブロックで、エラーメッセージを `thought` に表示するよう変更。

```typescript
} catch (err) {
  setThought(String(err));  // エラーも bubble に表示
}
```

**理由**: 未知コマンドや引数エラーのメッセージがユーザに見えるようになる。変更は1行のみ。

### 9. 重複登録ポリシー

**選択**: `CommandRouter::register` で同名コマンドが既に存在する場合、`warn!` ログを出力し後勝ちで上書きする。

**理由**: panic はアプリ全体を落とすため不適切。開発時に気づける程度の警告で十分。

## Risks / Trade-offs

- **[タイマー消失]** アプリ再起動でタイマーが失われる → 許容範囲（サンプル実装）。将来必要なら DB 永続化に移行可能。
- **[複数タイマー]** 同時に複数タイマーを設定した場合、完了通知が thought bubble を順次上書きする → v1 では許容。将来的には通知キューの導入を検討。
- **[ウィンドウ非表示]** ウィンドウが隠れている状態でタイマー完了すると通知を見逃す → v1 では許容。将来的には OS 通知との併用を検討。
- **[時間パースの曖昧さ]** `3` だけ入力された場合 → 「分」として扱い、usage で案内。
