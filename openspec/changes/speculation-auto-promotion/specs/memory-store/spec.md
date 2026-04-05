## ADDED Requirements

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
