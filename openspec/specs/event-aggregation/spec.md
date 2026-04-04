# Event Aggregation

## Purpose

LLMに送信するイベントを集約・制限し、プロンプトサイズを最適化する。

## Requirements

### Requirement: イベントを(app, title)単位で集約する
システムはLLMへのイベント送信前に、同一の(app, title)ペアを持つイベントをグルーピングし、duration_secondsを合計しなければならない（SHALL）。

#### Scenario: 同一アプリ・タイトルのイベント集約
- **WHEN** 同じappとtitleを持つ複数のイベントが存在する時
- **THEN** 1つのエントリにまとめ、duration_secondsを合計し、最も新しいタイムスタンプをlast_activeとして保持する

#### Scenario: 異なるタイトルのイベント
- **WHEN** 同じappだが異なるtitleを持つイベントが存在する時
- **THEN** それぞれ別のエントリとして集約する

### Requirement: 集約後のイベントを直近アクティブ順で上位30件に制限する
システムは集約後のイベントをlast_active（最も新しいタイムスタンプ）の降順でソートし、上位30件のみをLLMに渡さなければならない（SHALL）。

#### Scenario: 集約後30件以下の場合
- **WHEN** 集約後のイベント数が30件以下の時
- **THEN** すべてのイベントをLLMに渡す

#### Scenario: 集約後30件を超える場合
- **WHEN** 集約後のイベント数が30件を超える時
- **THEN** last_active降順で上位30件のみをLLMに渡し、残りは切り捨てる

### Requirement: LLMプロンプトでイベントをapp別にグルーピング表示する
システムはLLMプロンプト内で集約済みイベントをapp単位でグルーピングし、各appの合計duration_secondsとその内訳（title別）を二段構造で表示しなければならない（SHALL）。

#### Scenario: 複数アプリのイベントがある場合
- **WHEN** 集約済みイベントに複数のappが含まれる時
- **THEN** app別にグルーピングし、各appの合計秒数を表示した上で、その配下にtitle別の内訳を表示する
