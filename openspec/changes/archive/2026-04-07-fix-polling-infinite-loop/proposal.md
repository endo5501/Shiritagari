## Why

メインポーリングループで `continue` 文がループ末尾の `tokio::time::sleep()` をバイパスしており、新規イベントがない状態が続くと約3秒間隔の無限高速ポーリングが発生する。ActivityWatch APIへの不要な負荷とCPU/ネットワークリソースの浪費を引き起こす。

## What Changes

- ポーリングループのスリープ位置をループ末尾から先頭に移動し、`continue` によるスリープスキップを構造的に不可能にする
- 初回サイクルは `first_run` フラグでスリープをスキップし、起動直後のポーリング即時実行を維持する

## Capabilities

### New Capabilities

（なし）

### Modified Capabilities

- `activity-observation`: ポーリングループのスリープ制御を変更。「設定された間隔でイベントを定期取得する」という要件の実装方法が変わる（要件自体は変更なし）

## Impact

- `src-tauri/src/lib.rs` のメインポーリングループ（283行目付近）
- 既存のテストへの影響なし（ループ構造のみの変更、ポーリングロジック自体は不変）
