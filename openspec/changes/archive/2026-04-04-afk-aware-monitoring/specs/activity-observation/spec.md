## MODIFIED Requirements

### Requirement: ActivityWatch APIからイベントを定期取得する
システムは設定された間隔（デフォルト10分）でActivityWatch REST API (localhost:5600) からウィンドウイベントとAFKイベントを取得しなければならない（SHALL）。AFK状態に関わらずイベント取得と推論を実行しなければならない（SHALL）。

#### Scenario: 正常なポーリング
- **WHEN** ポーリング間隔が経過した時
- **THEN** SQLiteに保存された最終処理カーソル（タイムスタンプ）以降のイベントのみを `GET /api/0/buckets/aw-watcher-window_{hostname}/events?start={cursor}` と `GET /api/0/buckets/aw-watcher-afk_{hostname}/events?start={cursor}` で取得する

#### Scenario: ユーザがAFK状態の場合
- **WHEN** aw-watcher-afkのステータスが "afk" である時
- **THEN** イベント取得と推論は通常通り実行し、生成された質問はフロントエンドにemitせずメモリ上のキューに保存する

#### Scenario: 前回と同じイベント状況の場合
- **WHEN** 取得したイベントがすべて処理済みで新規イベントがない時
- **THEN** 推論処理をスキップする（AFK・非AFK問わず）
