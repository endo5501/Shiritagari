## 1. poller.rs の AFK ハンドリング変更

- [x] 1.1 `poll_once()` から AFK 時の early return を削除し、AFK 状態でもイベント取得を続行するよう変更する
- [x] 1.2 `PollResult` から `skipped_afk` フィールドを削除する（`is_afk` は残す）
- [x] 1.3 poller の既存テストを更新し、AFK 時でもイベントが返されることを検証する

## 2. メインループに質問キューと AFK 復帰検知を追加

- [x] 2.1 `lib.rs` のポーリングループに `pending_question: Option<String>` と `was_afk: bool` を追加する
- [x] 2.2 `skipped_afk` による `continue` を削除し、AFK 時も推論フローを通すよう変更する
- [x] 2.3 質問 emit 部分を分岐させる — 在席時は即 emit、AFK 時は `pending_question` に保存
- [x] 2.4 サイクル冒頭で AFK 復帰（`was_afk && !is_afk`）を検知し、`pending_question` があれば emit してクリアする
- [x] 2.5 サイクル終了時に `was_afk = is_afk` を更新する

## 3. テスト

- [x] 3.1 AFK 中にイベント取得・推論が実行されることのテスト
- [x] 3.2 AFK 中の質問が `pending_question` に保存されることのテスト
- [x] 3.3 AFK 復帰時に `pending_question` が emit されキューがクリアされることのテスト
- [x] 3.4 AFK 中に新イベントがなければ推論がスキップされることのテスト
