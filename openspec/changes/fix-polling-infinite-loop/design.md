## Context

メインポーリングループ (`src-tauri/src/lib.rs:283-398`) は `loop` 内で `poll_once()` → イベント処理 → `tokio::time::sleep()` の順で実行される。しかし、新規イベントがない場合やinference provider作成失敗時に `continue` でループ先頭に戻り、末尾の `sleep()` をスキップする。結果として、イベントが枯渇した状態（AFK継続、同一ウィンドウ作業など）で約3秒間隔の高速ポーリングが発生する。

## Goals / Non-Goals

**Goals:**
- `continue` 文がスリープをバイパスする構造的欠陥を解消する
- 起動直後の初回ポーリングは即時実行を維持する（デバッグ体験の確保）

**Non-Goals:**
- ポーリング間隔の動的調整（バックオフ等）
- graceful shutdown の実装
- ポーリングロジック自体のリファクタリング

## Decisions

### スリープをループ先頭に移動 + first_run フラグ（方針 B-1）

スリープをループ末尾から先頭に移動する。`first_run` フラグで初回のみスリープをスキップし、即時実行を維持する。

```rust
let mut first_run = true;
loop {
    if first_run {
        first_run = false;
    } else {
        tokio::time::sleep(poller.interval_duration()).await;
    }
    debug!("Polling cycle started");
    // ... (continue は自由に使える)
}
```

**却下した代替案:**

- **A. 各 `continue` 前にスリープ追加**: 変更は最小だが、今後 `continue` を追加する際に同じバグが再発するリスクがある
- **C. ループ本体を関数に抽出**: 構造的に安全だが、この修正に対してリファクタリング規模が大きすぎる

## Risks / Trade-offs

- [初回スキップの安全性] `first_run` フラグが `continue` で先頭に戻った時もスリープを保証する → `first_run` は初回ループの最初で即 `false` に設定されるため、2回目以降は必ずスリープを通る。安全。
