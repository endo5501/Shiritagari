## Context

現在 `send_message` は全入力を LLM チャットとして処理する。タイマーのような即座のアクションを実行する手段がなく、機能追加のたびに `send_message` を直接改修する必要がある。LLM Provider で既に trait ベースのプラグインパターンが確立されており、同じ手法をコマンドシステムに適用する。

## Goals / Non-Goals

**Goals:**
- `CommandPlugin` trait による拡張可能なコマンドシステム
- 新コマンド追加 = ファイル追加 + 登録の1行、で済む構造
- `send_message` 内での透過的なルーティング（フロントエンド変更不要）
- サンプルとして Timer / Help プラグインを実装

**Non-Goals:**
- 外部プラグインの動的ロード（WASM 等）
- 設定ファイルによるプラグインの有効/無効切り替え
- 繰り返しタイマー（ポモドーロ等）
- タイマーの永続化（アプリ再起動で消失して構わない）

## Decisions

### 1. コマンドルーティングはバックエンド（Rust）側で行う

**選択**: `send_message` 内で `/` プレフィックスを検出し、`CommandRouter` にディスパッチ。マッチしなければ従来のチャット処理にフォールバック。

**理由**: UI 実装に依存しないため、フロントエンドの刷新時に影響を受けない。Rust 側でシステムリソース（タイマー、DB、通知）に直接アクセスできる。

**代替案**: フロントエンドで `/` を判定し別の Tauri コマンドを呼ぶ方式。関心の分離はきれいだがフロント変更が必要になり、ユーザの方針に反する。

### 2. CommandPlugin trait の設計

```rust
#[async_trait]
pub trait CommandPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn usage(&self) -> &str;
    async fn execute(&self, args: &str, ctx: &CommandContext) -> Result<CommandResult, String>;
}
```

**選択**: LLM Provider trait と同じパターン。`CommandContext` 経由で `AppHandle`（emit 用）と `Database` を渡す。

**理由**: 既存の `LlmProvider` trait パターンと一貫性がある。`CommandContext` にまとめることで、将来的なコンテキスト拡張が容易。

### 3. タイマーは tokio::spawn で管理

**選択**: `/timer 3時間` → `tokio::spawn(async { sleep(duration).await; emit(...) })` でワンショット実行。

**理由**: サンプル実装として最小限。永続化不要（アプリ再起動で消失許容）。DB やポーリングループへの依存がなくシンプル。

**代替案**: DB に保存 + ポーリングでチェック。再起動耐性があるが、今の段階ではオーバーエンジニアリング。

### 4. 時間パーサーの設計

日本語の時間表現（`3時間`, `30分`, `1時間30分`, `90秒`）を `Duration` に変換する。正規表現で `(\d+)時間`, `(\d+)分`, `(\d+)秒` をそれぞれ抽出し合算する。

### 5. CommandRouter の AppState への配置

**選択**: `AppState` に `CommandRouter` を `Arc<CommandRouter>` として保持。`run()` 内で各プラグインを登録してから AppState に渡す。

**理由**: `send_message` から `State<AppState>` 経由で自然にアクセスできる。

## Risks / Trade-offs

- **[タイマー消失]** アプリ再起動でタイマーが失われる → 許容範囲（サンプル実装）。将来必要なら DB 永続化に移行可能。
- **[コマンド名衝突]** 複数プラグインが同名コマンドを登録する可能性 → `CommandRouter::register` 時に重複チェックして panic/warn。
- **[時間パースの曖昧さ]** `3` だけ入力された場合の解釈 → 数値のみの場合は「分」として扱い、usage メッセージで案内する。
