## ADDED Requirements

### Requirement: パターン記憶の管理
システムはパターン記憶（長期）をSQLiteに保存し、trigger条件、meaning、confidence、最終確認日を管理しなければならない（SHALL）。ページング対応のため、件数取得と limit/offset 付きクエリを提供しなければならない（SHALL）。

#### Scenario: パターン記憶の作成（昇格）
- **WHEN** 同一のtrigger条件に対するエピソード記憶が3件以上蓄積された時
- **THEN** 新しいパターン記憶を作成し、元のエピソードからtrigger・meaningを集約し、初期confidenceを設定する

#### Scenario: パターン記憶のconfidenceマッチ時更新
- **WHEN** ポーリングで取得した操作が既存パターンにマッチした時
- **THEN** そのパターンのconfidenceを微増させ、last_confirmedを現在日時に更新する

#### Scenario: パターン記憶のconfidence時間経過減衰
- **WHEN** パターンのconfidenceを評価する時
- **THEN** `effective_confidence = base_confidence × 0.99 ^ days_since_last_confirmed` で実効confidenceを算出する

#### Scenario: パターン記憶のソフトデリート
- **WHEN** パターンの実効confidenceが0.3以下になった時
- **THEN** そのパターン記憶にdeleted_atタイムスタンプを記録し、ソフトデリート状態とする（検索対象から除外されるが、データは保持される）

#### Scenario: ソフトデリートされたパターンの復元
- **WHEN** ソフトデリート状態のパターンと同一triggerに対する新しいエピソードが蓄積された時
- **THEN** deleted_atをクリアし、confidenceを再設定してパターンを復元する

#### Scenario: ソフトデリートされたパターンの完全削除
- **WHEN** ソフトデリートから30日が経過し、復元されなかった時
- **THEN** そのパターン記憶を完全に削除する

#### Scenario: アクティブパターンのページング取得
- **WHEN** limit と offset を指定してアクティブパターンを取得する時
- **THEN** deleted_at が NULL のパターンを指定範囲で返し、総件数も取得できる

### Requirement: エピソード記憶の管理
システムはエピソード記憶（中期）をSQLiteに保存し、タイムスタンプ、コンテキスト（操作内容）、質問、回答、タグを管理しなければならない（SHALL）。ページング対応のため、件数取得と limit/offset 付きクエリを提供しなければならない（SHALL）。

#### Scenario: エピソード記憶の保存
- **WHEN** ユーザが質問に回答した時
- **THEN** 質問時のコンテキスト（app、title、duration）、質問文、回答文、生成されたタグを含むエピソード記憶を保存する

#### Scenario: エピソード記憶の自動削除
- **WHEN** エピソード記憶の作成から1ヶ月が経過した時
- **THEN** そのエピソード記憶を自動削除する

#### Scenario: エピソードのページング取得
- **WHEN** limit と offset を指定してエピソードを取得する時
- **THEN** エピソードを新しい順で指定範囲で返し、総件数も取得できる

### Requirement: 推測ログの管理
システムは推測ログ（短期）をSQLiteに保存し、推測内容、confidence、確認済みフラグ、有効期限を管理しなければならない（SHALL）。

#### Scenario: 推測ログの保存
- **WHEN** LLM推論が完了した時
- **THEN** 推測内容、confidence、タイムスタンプ、有効期限（3日後）を含む推測ログを保存する

#### Scenario: 推測ログの自動削除
- **WHEN** 推測ログの有効期限を過ぎた時
- **THEN** その推測ログを自動削除する

### Requirement: ユーザプロファイルの管理
システムはユーザプロファイル（職業、スキル、関心事等）をSQLiteに保存し、LLM推論のコンテキストとして利用しなければならない（SHALL）。

#### Scenario: プロファイル更新
- **WHEN** チャット対話やエピソード記憶からユーザの属性情報が得られた時
- **THEN** ユーザプロファイルを更新する

### Requirement: Speculation からの自動パターン昇格
システムは同一の observed_app + observed_title を持つ Speculation が6件以上蓄積された場合、自動的に Pattern に昇格しなければならない（SHALL）。昇格時の meaning は最新の Speculation の inference テキストを使用する。

#### Scenario: 昇格条件を満たした場合
- **WHEN** 同一の observed_app と observed_title を持つ Speculation が6件以上存在する時
- **THEN** 最新の Speculation の inference を meaning として使用し、Pattern に昇格する

#### Scenario: 昇格条件を満たさない場合
- **WHEN** 同一の observed_app と observed_title を持つ Speculation が6件未満の時
- **THEN** 昇格は行わない

#### Scenario: 既にアクティブな Pattern が存在する場合
- **WHEN** 昇格対象の trigger と同一の Pattern が既に存在する時
- **THEN** 新規 Pattern は作成せず、既存 Pattern をそのまま維持する

#### Scenario: ソフトデリートされた Pattern が存在する場合
- **WHEN** 昇格対象の trigger と同一のソフトデリート済み Pattern が存在する時
- **THEN** そのパターンを復元し、confidence を再設定する

#### Scenario: 定期実行
- **WHEN** run_cleanup が実行される時（毎ポーリングサイクル）
- **THEN** Speculation の集計と昇格チェックを実行する
