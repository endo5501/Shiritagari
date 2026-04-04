## ADDED Requirements

### Requirement: ログシステムの初期化
アプリケーション起動時に `env_logger` を初期化し、`RUST_LOG` 環境変数によるログレベル制御を有効にしなければならない（SHALL）。`RUST_LOG` 未設定時はログを出力しない。

#### Scenario: RUST_LOG=debug で起動
- **WHEN** `RUST_LOG=debug npm run tauri dev` で起動する
- **THEN** debug レベル以上のすべてのログがターミナルに出力される

#### Scenario: RUST_LOG 未設定で起動
- **WHEN** `RUST_LOG` を設定せずに起動する
- **THEN** ログは出力されず、既存の動作に影響しない

### Requirement: ポーリングサイクル開始ログ
各ポーリングサイクルの開始時に debug レベルでログを出力しなければならない（SHALL）。

#### Scenario: サイクル開始
- **WHEN** 10分タイマーが発火してポーリングサイクルが開始する
- **THEN** サイクル開始を示す debug ログが出力される

### Requirement: ActivityWatch 接続状態ログ
ActivityWatch への接続確認結果をログに出力しなければならない（SHALL）。接続不可の場合は warn レベル、接続可能の場合は debug レベルとする。

#### Scenario: ActivityWatch が利用不可
- **WHEN** ActivityWatch が起動していない、または接続できない
- **THEN** warn レベルで接続不可を示すログが出力される

#### Scenario: ActivityWatch が利用可能
- **WHEN** ActivityWatch に正常に接続できる
- **THEN** debug レベルで接続可能を示すログが出力される

### Requirement: AFK 判定結果ログ
AFK（離席）判定の結果をログに出力しなければならない（SHALL）。AFK の場合は info レベルとする。

#### Scenario: ユーザが離席中
- **WHEN** AFK バケットの判定結果が離席中である
- **THEN** info レベルで AFK スキップを示すログが出力される

#### Scenario: ユーザがアクティブ
- **WHEN** AFK バケットの判定結果がアクティブである
- **THEN** debug レベルでアクティブ状態を示すログが出力される

### Requirement: イベント取得結果ログ
ActivityWatch から取得したイベントの件数をログに出力しなければならない（SHALL）。新規イベント数とフィルタ後のイベント数の両方を含む。

#### Scenario: 新規イベントあり
- **WHEN** カーソル以降に新規イベントが存在する
- **THEN** debug レベルで取得件数とフィルタ後の件数が出力される

#### Scenario: 新規イベントなし
- **WHEN** カーソル以降に新規イベントが存在しない
- **THEN** debug レベルでイベント0件であることが出力される

### Requirement: プライバシーフィルタ結果ログ
プライバシーフィルタの適用結果をログに出力しなければならない（SHALL）。フィルタ前後の件数を含む。

#### Scenario: 一部のイベントが除外される
- **WHEN** プライバシーフィルタにより一部のイベントが除外される
- **THEN** debug レベルでフィルタ前の件数、フィルタ後の件数が出力される

#### Scenario: 全イベントが除外される
- **WHEN** プライバシーフィルタにより全イベントが除外される
- **THEN** info レベルで全イベントが除外されたことが出力される

### Requirement: パターンマッチング結果ログ
パターンマッチングの判定結果をログに出力しなければならない（SHALL）。マッチしたパターンのアプリ名、実効信頼度、決定されたアクションを含む。

#### Scenario: パターンが Silent と判定
- **WHEN** 既知パターンの実効信頼度が閾値以上で Silent と判定される
- **THEN** info レベルでアプリ名、実効信頼度、Silent アクションが出力される

#### Scenario: パターンが ReAsk と判定
- **WHEN** 既知パターンの実効信頼度が中程度で ReAsk と判定される
- **THEN** info レベルでアプリ名、実効信頼度、ReAsk アクションが出力される

#### Scenario: パターンマッチなし
- **WHEN** いずれの既知パターンにもマッチしない
- **THEN** info レベルでパターンマッチなし、LLM 推論に進むことが出力される

### Requirement: LLM プロバイダー生成結果ログ
LLM プロバイダーの生成成功/失敗をログに出力しなければならない（SHALL）。失敗時はエラー内容を含む。

#### Scenario: プロバイダー生成成功
- **WHEN** LLM プロバイダーが正常に生成される
- **THEN** debug レベルでプロバイダー種別と生成成功が出力される

#### Scenario: プロバイダー生成失敗
- **WHEN** LLM プロバイダーの生成に失敗する
- **THEN** warn レベルでエラー内容を含む失敗ログが出力される

### Requirement: LLM 推論結果ログ
LLM 推論の結果をログに出力しなければならない（SHALL）。confidence 値と should_ask の判定結果を含む。LLM レスポンスの全文は出力しない。

#### Scenario: LLM 推論成功
- **WHEN** LLM 推論が正常に完了する
- **THEN** info レベルで confidence 値、should_ask、決定されたアクションが出力される

#### Scenario: LLM 推論失敗
- **WHEN** LLM 推論がエラーとなる
- **THEN** warn レベルでエラー概要が出力される

### Requirement: 質問送信ログ
フロントエンドへの質問送信時にログを出力しなければならない（SHALL）。

#### Scenario: 質問が送信される
- **WHEN** `shiritagari-question` イベントが emit される
- **THEN** info レベルで質問送信を示すログが出力される

### Requirement: サイクル完了ログ
ポーリングサイクルの完了時に結果サマリを debug レベルで出力しなければならない（SHALL）。

#### Scenario: サイクル正常完了
- **WHEN** ポーリングサイクルが正常に完了し、イベントが acknowledge される
- **THEN** debug レベルでサイクル完了と処理結果のサマリが出力される

### Requirement: LLM呼び出し詳細ログ
LLM呼び出しの前後で、プロンプトサイズと処理状況をログに出力しなければならない（SHALL）。

#### Scenario: LLM呼び出し開始時
- **WHEN** LLM推論のためにプロバイダの `infer()` を呼び出す直前
- **THEN** info レベルで集約後イベント数を出力し、debug レベルでプロンプト文字数を出力する

#### Scenario: LLM呼び出し完了時
- **WHEN** LLM推論が正常に完了した時
- **THEN** info レベルで推論完了と所要時間を出力する

#### Scenario: LLM呼び出し失敗時
- **WHEN** LLM推論がエラーとなった時
- **THEN** warn レベルでエラー内容と所要時間を出力する
