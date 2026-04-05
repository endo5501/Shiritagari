## 1. DB クエリ追加

- [x] 1.1 `memory/speculations.rs` に `get_speculation_promotion_candidates(min_count)` を追加し、テストを作成。同一 (observed_app, observed_title) で GROUP BY + HAVING COUNT >= min_count、各グループの最新 inference を返す

## 2. 昇格ロジック

- [x] 2.1 `memory/cleanup.rs` に `promote_speculations_to_patterns(required_count)` を追加し、テストを作成。候補を取得し、既存の `try_promote_to_pattern` で昇格を実行する
- [x] 2.2 `run_cleanup` 内で `promote_speculations_to_patterns(6)` を呼び出し、`CleanupResult` に昇格件数を追加

## 3. 統合テスト

- [x] 3.1 `cleanup.rs` に Speculation 6件で Pattern 昇格が発生する統合テストを追加
- [x] 3.2 `cleanup.rs` に既存 Pattern がある場合に重複昇格しないテストを追加
