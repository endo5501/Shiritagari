## Why

現状、知識（Pattern）の蓄積はユーザがAIの質問に回答した場合（Episode経由）にのみ発生する。LLMが「聞くまでもない」と判断した行動（Silent）や、ユーザが質問を無視した場合は、Speculationとして3日間記録されるが、Patternには昇格せず消滅する。これにより、繰り返し観測される行動パターンが知識として定着しない問題がある。

## What Changes

- 同一の `observed_app` + `observed_title` を持つ Speculation が一定回数（6件）以上蓄積された場合、ユーザ確認なしで自動的に Pattern に昇格する
- 昇格時の meaning は最新の Speculation の inference テキストを使用
- 昇格チェックは既存の `run_cleanup` 内で毎ポーリングサイクルごとに実行

## Capabilities

### New Capabilities
（なし）

### Modified Capabilities
- `memory-store`: Speculation からの自動パターン昇格ロジックを追加

## Impact

- **Rust コード**: `memory/speculations.rs` に集計クエリ追加、`memory/cleanup.rs` に昇格チェック追加
- **既存動作への影響**: Speculation の保存・削除ロジックは変更なし。既存の `try_promote_to_pattern` を再利用するため、既存パターンとの重複チェックやソフトデリート復元も自動的に適用される
