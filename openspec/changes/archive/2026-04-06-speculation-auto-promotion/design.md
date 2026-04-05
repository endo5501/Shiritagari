## Context

Speculation は毎ポーリングサイクル（デフォルト10分）で LLM 推論結果として保存され、3日で期限切れ削除される。Pattern への昇格は現在 Episode 経由（ユーザ回答が必要、閾値3件）のみ。`run_cleanup` は毎ポーリングサイクルで実行される。

## Goals / Non-Goals

**Goals:**
- 同一 trigger (app + title) の Speculation が6件以上ある場合、自動で Pattern に昇格する
- 既存の `try_promote_to_pattern` を再利用し、重複チェック・ソフトデリート復元を自動的に適用

**Non-Goals:**
- Speculation の保存・削除ロジックの変更
- LLM による inference テキストの類似度判定（app + title ベースで十分）
- 昇格頻度の制御（毎 cleanup で実行、過剰な場合は将来検討）

## Decisions

### 1. 同一パターンの判定: app + title ベース
- **選択**: `observed_app` + `observed_title` の完全一致でグループ化
- **理由**: Episode の昇格ロジック（`count_episodes_by_app_and_title`）と同じアプローチ。inference テキストの類似度判定は複雑すぎる
- **代替案**: LLM による類似度判定（精度は高いがコストが大きい）

### 2. 昇格閾値: 6件
- **選択**: 同一 trigger の Speculation が6件以上
- **理由**: Episode 昇格（3件）の倍。Speculation はユーザ確認なしで自動生成されるため、より高い閾値で誤昇格を防ぐ

### 3. meaning の取得: 最新の inference テキスト
- **選択**: 同一 trigger グループ内で最も新しい Speculation の inference を使用
- **理由**: 最新の推論が最も文脈を反映している

### 4. 実行タイミング: run_cleanup 内
- **選択**: 既存の `run_cleanup` メソッドに昇格チェックを追加
- **理由**: 追加のタイマーや実行スケジュール不要。cleanup は毎ポーリングサイクルで既に実行されている

### 5. 集計クエリ: SQL で GROUP BY + HAVING
- **選択**: `speculations` テーブルを `observed_app, observed_title` で集計し、COUNT >= 6 のグループを抽出。各グループの最新 inference を取得
- **理由**: 1クエリで候補リストを取得でき、Rust 側のロジックを最小化できる

## Risks / Trade-offs

- **[誤った推論の定着]** → ユーザ確認なしで Pattern 化するため、LLM の誤推論が定着するリスクがある → confidence の時間経過減衰により、確認されないパターンは自然にソフトデリートされる（0.99^日数で減衰、0.3以下で削除）
- **[title の揺れ]** → ウィンドウタイトルが微妙に変わる場合（ファイル名の違い等）、同一行動でも別グループになる → 現時点では許容。将来的に部分一致や正規化を検討
