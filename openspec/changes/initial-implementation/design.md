## Context

Shiritagariは新規プロジェクトとしてゼロから構築する常駐型AIエージェントアプリケーション。ユーザのPC操作をActivityWatch経由で観察し、LLMで行動を推論し、不明な行動について質問し、回答を記憶して学習するループを実現する。

前提条件:
- ActivityWatch がローカル(localhost:5600)で稼働していること
- LLM APIキーが設定されていること（ローカルLLM使用時は不要）
- macOS / Windows の両環境で動作すること

## Goals / Non-Goals

**Goals:**
- ActivityWatch APIから定期的にユーザの操作情報を取得できる
- 3層ハイブリッド記憶モデルをSQLiteで管理できる
- LLMによる行動推論とconfidenceベースの質問判定ができる
- Tauriの常駐チャットウィンドウでユーザと対話できる
- 複数のLLMプロバイダを設定で切替できる

**Non-Goals:**
- キャラクター表示（将来対応）
- ActivityWatch以外のデータソース連携
- モバイル対応
- クラウド同期・マルチデバイス対応
- ブラウザ拡張連携

## Decisions

### 1. アプリケーションフレームワーク: Tauri v2

**選択**: Tauri v2 (Rust backend + React/TypeScript frontend)

**代替案**:
- Electron: クロスプラットフォーム実績は豊富だが、メモリ消費が大きく常駐アプリには不向き
- Flutter Desktop: デスクトップサポートがまだ成熟途上

**理由**: 常駐アプリのためメモリフットプリントの小ささが重要。RustバックエンドによりSQLite操作やバックグラウンドポーリングも効率的に実装できる。将来のキャラクター表示にもWebView (React) で対応可能。

### 2. 記憶ストレージ: SQLite

**選択**: SQLite（単一ファイルDB）

**代替案**:
- JSON ファイル: シンプルだがクエリ性能に難あり
- ベクトルDB (ChromaDB等): 類似検索には強いがオーバーキル、依存も増える

**理由**: Tauriとの相性が良く、構造化クエリが容易。クロスプラットフォームで問題なし。類似エピソード検索はLLMに委ねることでベクトルDBなしで実現する。

### 3. LLM Provider抽象化

**選択**: Rust traitベースの抽象化 + 設定ファイル(TOML)で切替

```
trait LlmProvider {
    async fn infer(&self, input: InferenceInput) -> InferenceOutput;
    async fn chat(&self, messages: Vec<Message>) -> ChatResponse;
}
```

初期実装プロバイダ: Claude API, OpenAI API, Ollama

**理由**: ユーザの環境・予算に応じて選択可能にする。推論用（ポーリング判定、コスト重視）と対話用（チャット、品質重視）で別モデルを指定可能とする。

### 4. ポーリングアーキテクチャ

**選択**: Rustバックエンドでのtokioベースの定期タスク

```
[polling loop]
  → SQLiteからバケット別の最終処理カーソル(last_event_timestamp)を読み込む
  → GET localhost:5600/api/0/buckets/aw-watcher-window_{hostname}/events?start={cursor}
  → GET localhost:5600/api/0/buckets/aw-watcher-afk_{hostname}/events?start={cursor}
  → イベントIDによる重複排除（DB側UNIQUE制約 + UPSERT）
  → パターン記憶照合
  → (必要なら) LLM推論
  → (必要なら) フロントエンドへ質問イベント送信
  → カーソル更新をイベント処理とトランザクションで一括コミット
```

ポーリング間隔は設定ファイルで調整可能（デフォルト10分）。AFKステータスを確認し、離席中はスキップする。

**冪等性の保証**: バケットごとにSQLiteに永続化されたカーソル（最終処理イベントのタイムスタンプ）を保持し、取得範囲を明示的に制限する。カーソル更新とイベント処理を単一トランザクションで行うことで、再起動や障害時の重複処理を防止する。

### 5. フロントエンド・バックエンド間通信

**選択**: Tauri v2のIPC（invoke / event system）

- バックエンド→フロントエンド: Tauri event（質問通知、推論結果）
- フロントエンド→バックエンド: Tauri invoke（チャットメッセージ送信、設定変更）

### 6. confidence減衰モデル

**選択**: 指数減衰

```
effective_confidence = base_confidence × decay_rate ^ days_since_last_confirmed
```

- `decay_rate`: 0.99（デフォルト、設定可能）
- 閾値（質問判定の統一ステートマシン）:
  - THRESHOLD_SILENT (0.8): これ以上なら質問しない（推測ログに記録のみ）
  - 0.8未満: 新規推論の場合はユーザに質問する
  - THRESHOLD_RE_ASK (0.5): 既存パターンが減衰してこの値以下になった場合、再質問候補とする
  - THRESHOLD_SOFT_DELETE (0.3): これ以下でパターンをソフトデリート（deleted_atを記録、30日間の復元期間）
  - 復元期間(30日)を過ぎたソフトデリート済みパターンを完全削除

### 7. プロジェクト構成

```
shiritagari/
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── polling/     # ActivityWatch ポーリング
│   │   ├── memory/      # SQLite記憶モデル
│   │   ├── inference/   # LLM推論エンジン
│   │   ├── providers/   # LLM Provider実装
│   │   └── config.rs    # 設定管理
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                 # React frontend
│   ├── App.tsx
│   ├── components/
│   │   ├── ChatWindow.tsx
│   │   ├── MessageBubble.tsx
│   │   └── InputBar.tsx
│   └── hooks/
│       └── useTauriEvents.ts
├── config.toml          # ユーザ設定
└── package.json
```

## Risks / Trade-offs

**[ActivityWatch未起動時の動作]** → 起動時にActivityWatch接続を確認し、未起動時はポーリングを無効化してチャットのみモードで動作。接続復帰時に自動再開。

**[LLM APIコスト]** → ポーリング間隔の調整、パターン記憶のヒット率向上による呼び出し削減で対処。推論用に安価なモデルを選択可能にする。

**[質問の適切さ]** → 初期は質問が多すぎる/的外れになる可能性がある。confidence閾値とポーリング間隔を調整可能にしておき、ユーザフィードバックで改善する。「今は忙しい」等の拒否にも対応し、一定時間質問を控えるクールダウン機能を設ける。

**[プライバシー]** → 以下の技術的ガードレールで対処:
- **デフォルトはローカルLLMモード**: 初回起動時はOllamaなどローカルプロバイダをデフォルトとし、外部API使用は明示的なオプトインを必要とする
- **アプリ許可リスト/拒否リスト**: config.tomlでモニタリング対象のアプリを制御可能にする（allowlist/blocklist方式）。blocklist対象のアプリのウィンドウタイトルはLLMに送信しない
- **リダクションパイプライン**: LLMへの送信前に、設定されたパターン（メールアドレス、URL内のトークン等）を自動マスキングする
- すべてのデータはローカルSQLiteに保存。外部API利用時はどのデータが送信されるかをユーザに明示する
