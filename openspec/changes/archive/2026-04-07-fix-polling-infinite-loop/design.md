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

### `tokio::time::interval` + `MissedTickBehavior::Delay` でループ先頭にスリープを配置

`tokio::time::interval` はループ先頭に `tick().await` を置くパターンで、初回は即時返却、以降はインターバル分待つ。`MissedTickBehavior::Delay` を設定し、サイクルがインターバルを超過した場合もバーストせず、完了後に必ずフルインターバル待つようにする。

```rust
let mut ticker = tokio::time::interval(poller.interval_duration());
ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

loop {
    ticker.tick().await;
    debug!("Polling cycle started");
    // ... (continue は自由に使える、必ず tick().await を通る)
}
```

**却下した代替案:**

- **A. 各 `continue` 前にスリープ追加**: 変更は最小だが、今後 `continue` を追加する際に同じバグが再発するリスクがある
- **B-1. `first_run` フラグ + `tokio::time::sleep`**: 構造的に安全だが、`tokio::time::interval` が同じ振る舞いを標準で提供しているため不要
- **C. ループ本体を関数に抽出**: 構造的に安全だが、この修正に対してリファクタリング規模が大きすぎる

## Risks / Trade-offs

- [MissedTickBehavior] デフォルトの `Burst` モードではサイクル超過時に即座に次の tick が発火するため、`Delay` を明示的に設定する必要がある。`Delay` により「前回 tick 完了からインターバル分待つ」動作が保証される。
