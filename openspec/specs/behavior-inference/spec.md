## ADDED Requirements

### Requirement: LLMによる行動推論
システムはActivityWatchイベントと記憶ストアの情報をLLMに渡し、ユーザの行動を推論しなければならない（SHALL）。推論結果のinferenceフィールドはデスクトップマスコットの吹き出しに表示されるため、短い独り言風テキスト（40文字以内）で出力しなければならない（SHALL）。

#### Scenario: パターンにマッチしない操作の推論
- **WHEN** ポーリングで取得した操作が既存パターンにマッチしない時
- **THEN** 直近のイベントログ、関連するエピソード記憶、ユーザプロファイルをLLMに送信し、推論結果（inference, confidence, should_ask, suggested_question）を取得する

#### Scenario: パターンにマッチする操作
- **WHEN** ポーリングで取得した操作が既存パターンにマッチする時
- **THEN** LLM呼び出しをスキップし、パターンのmeaningを推論結果として使用する

#### Scenario: inferenceフィールドの出力形式
- **WHEN** LLMが推論結果を生成する時
- **THEN** inferenceフィールドは40文字以内の独り言風テキストとする（例:「Reactのコンポーネントを書いてるな...」）

### Requirement: confidenceベースの質問判定
システムはconfidence値に基づく統一ステートマシンでユーザへの質問要否を判定しなければならない（SHALL）。

#### Scenario: 高confidence — 質問しない (≥0.8)
- **WHEN** 推論結果のconfidenceが0.8以上の時
- **THEN** ユーザに質問せず、推測ログに記録のみ行う

#### Scenario: 低confidence — 新規推論で質問する (<0.8)
- **WHEN** 新規推論（パターン未マッチ）でconfidenceが0.8未満の時
- **THEN** LLMが生成したsuggested_questionをユーザに提示する

#### Scenario: 既存パターン減衰 — 再質問候補 (≤0.5)
- **WHEN** 既存パターンの実効confidenceが0.5以下に減衰した時
- **THEN** そのパターンのmeaningが現在も正しいか再確認する質問を生成し、ユーザに提示する

### Requirement: 推論結果のフロントエンド送信
システムは推論結果を毎サイクルフロントエンドに送信しなければならない（SHALL）。

#### Scenario: LLM推論完了時のイベント送信
- **WHEN** LLMによる推論が完了した時
- **THEN** `shiritagari-thought` イベントとしてinferenceテキストとconfidence値をフロントエンドに送信する

#### Scenario: 質問時のイベント送信
- **WHEN** 推論結果でshould_askがtrueの時
- **THEN** `shiritagari-question` イベントで質問を送信し（既存通り）、加えてウィンドウを最前面に設定する

#### Scenario: パターンマッチ（Silent）時
- **WHEN** 既知パターンにマッチして質問不要の時
- **THEN** `shiritagari-thought` イベントは送信しない（変化がないため）

### Requirement: 1回のポーリングで最大1質問
システムは1回のポーリングサイクルで最大1つの質問のみユーザに提示しなければならない（SHALL）。

#### Scenario: 複数の不明操作がある場合
- **WHEN** 1回のポーリングで複数の不明操作が検出された時
- **THEN** 最もconfidenceが低い操作について1つだけ質問する
