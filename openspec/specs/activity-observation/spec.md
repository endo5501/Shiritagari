## ADDED Requirements

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

### Requirement: ポーリングの冪等性を保証する
システムはイベント処理の重複を防止するため、バケットごとの処理カーソルを永続化し、冪等性を保証しなければならない（SHALL）。

#### Scenario: カーソルのトランザクション更新
- **WHEN** 取得したイベントの処理が完了した時
- **THEN** カーソル更新とイベント処理結果（推測ログ等）を単一のSQLiteトランザクションで一括コミットする

#### Scenario: アプリ再起動後のポーリング
- **WHEN** アプリが再起動された時
- **THEN** SQLiteに保存された最終処理カーソルから再開し、処理済みイベントを再処理しない

#### Scenario: イベントIDによる重複排除
- **WHEN** ActivityWatch APIから取得したイベントが既に処理済みの場合
- **THEN** DB側のUNIQUE制約により重複を排除し、下流の記憶更新を発生させない

### Requirement: ActivityWatch未接続時にグレースフルに動作する
ActivityWatchが起動していない場合、システムはポーリングを無効化し、チャットのみモードで動作しなければならない（SHALL）。

#### Scenario: ActivityWatch未起動での起動
- **WHEN** アプリ起動時にActivityWatch (localhost:5600) に接続できない時
- **THEN** ポーリングを無効化し、チャットのみモードで動作する

#### Scenario: ActivityWatch接続復帰
- **WHEN** 接続チェックでActivityWatchの応答を検出した時
- **THEN** ポーリングを自動的に再開する

### Requirement: ポーリング間隔を設定可能にする
ユーザは設定ファイル(config.toml)でポーリング間隔を変更できなければならない（SHALL）。

#### Scenario: カスタムポーリング間隔
- **WHEN** config.tomlで `polling_interval_minutes = 30` と設定されている時
- **THEN** 30分間隔でActivityWatch APIをポーリングする
