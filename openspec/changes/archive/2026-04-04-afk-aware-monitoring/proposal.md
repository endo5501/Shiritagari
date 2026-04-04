## Why

現在、ポーリング発火時にユーザがAFKだと推論処理を完全にスキップしている。ポーリング間隔とAFKタイミングが重なると長時間モニタリングが行われない。AFK中でもAFK前の操作は推論する価値があり、質問をキューイングしてユーザ復帰時に提示すべきである。

## What Changes

- `poll_once()` がAFK状態に関わらずイベントを取得するよう変更
- AFK中に生成された質問をメモリ上のキュー（`Option<String>` — 最新1件のみ保持）に保存
- ユーザがAFKから復帰したことを次のポーリングサイクルで検知し、キューの質問をemitする
- 前回と同じイベント状況ならスキップする既存ロジックにより、AFK中の重複推論を自然に防止

## Capabilities

### New Capabilities

- `question-queueing`: AFK中に生成された質問をメモリ上にキューイングし、ユーザ復帰時にemitする機能

### Modified Capabilities

- `activity-observation`: AFK時の動作要件を変更 — 推論スキップから「イベント取得・推論は実行し質問のみ遅延」へ

## Impact

- `src-tauri/src/polling/poller.rs`: `poll_once()` のAFKハンドリング変更
- `src-tauri/src/lib.rs`: メインポーリングループに質問キューとAFK復帰検知を追加
- 既存テストのAFK関連テストケース更新が必要
