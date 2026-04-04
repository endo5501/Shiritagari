## Why

`poll_once()` のページネーションループで、ActivityWatch APIの `start` パラメータがinclusive（指定時刻を含む）であるため、最後のイベントのタイムスタンプを次ページのcursorに使うと同じイベントが再度返され、イベントが100件以上ある場合に無限ループが発生する。

## What Changes

- ページネーションのcursor更新ロジックを修正し、最後のイベントのタイムスタンプに1ミリ秒加算して次ページのstartとすることで重複取得を防止する
- 安全策としてページネーションに最大ループ回数の上限を設ける

## Capabilities

### New Capabilities

なし

### Modified Capabilities

- `activity-observation`: ページネーション時のcursor更新ロジックの修正（イベント取得の内部実装変更だが、重複イベント取得を防止するため要件レベルの動作が変わる）

## Impact

- `src-tauri/src/polling/poller.rs`: ページネーションループのcursor更新ロジック修正
