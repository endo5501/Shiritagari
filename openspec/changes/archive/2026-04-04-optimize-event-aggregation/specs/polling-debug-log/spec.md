## ADDED Requirements

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
