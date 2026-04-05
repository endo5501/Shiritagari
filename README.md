# Shiritagari

ユーザのPC操作を能動的に観察・質問・学習する常駐型AIエージェント。

[ActivityWatch](https://activitywatch.net/)と連携してウィンドウ操作を定期的にモニタリングし、不明な行動についてLLMで推論・質問し、回答を記憶して学習するループを実現します。

## 前提条件

- [Node.js](https://nodejs.org/) v18+
- [Rust](https://rustup.rs/) (stable)
- [ActivityWatch](https://activitywatch.net/) がインストール・起動済み（localhost:5600）
- LLMプロバイダ（下記いずれか）:
  - [Ollama](https://ollama.ai/)（デフォルト、ローカル実行）
  - Claude API（`ANTHROPIC_API_KEY` 環境変数）
  - OpenAI API（`OPENAI_API_KEY` 環境変数）

## セットアップ

```bash
# 依存パッケージのインストール
npm install

# Rust側の依存確認（初回は時間がかかります）
cd src-tauri && cargo check && cd ..
```

## 開発

```bash
# 開発モードで起動（ホットリロード対応）
npm run tauri dev

# デバッグログ付きで起動（ポーリング判定過程を表示）
RUST_LOG=debug npm run tauri dev

# 重要な判定結果のみ表示
RUST_LOG=info npm run tauri dev

# 特定モジュールのみ
RUST_LOG=shiritagari_app_lib::polling=debug npm run tauri dev
```

## テスト

```bash
# 全テスト一括実行（型チェック + フロントエンド + Rust）
npm run test:all

# 個別実行
npm test                  # フロントエンドテスト（vitest）
npm run test:rust         # Rustユニットテスト
npm run test:typecheck    # TypeScript型チェック
npm run test:watch        # フロントエンドテスト（watchモード）
```

## ビルド

```bash
# プロダクションビルド（OS別のバイナリを生成）
npm run tauri build
```

生成物は `src-tauri/target/release/bundle/` に出力されます。

## 設定

設定ファイルは以下の場所に作成します（なければデフォルト値で動作）:

- macOS: `~/Library/Application Support/shiritagari/config.toml`
- Windows: `%APPDATA%\shiritagari\config.toml`

```toml
[polling]
interval_minutes = 10          # ポーリング間隔（デフォルト: 10分）

[llm]
provider = "ollama"            # "ollama" | "claude" | "openai"
model = "llama3"               # 使用モデル
api_key_env = "ANTHROPIC_API_KEY"  # APIキーの環境変数名

# 推論用と対話用で別モデルを指定可能
# inference_provider = "ollama"
# inference_model = "llama3"
# chat_provider = "claude"
# chat_model = "claude-sonnet-4-20250514"

[privacy]
blocklist_apps = ["Signal", "1Password"]  # モニタリング除外アプリ
# allowlist_apps = ["Chrome", "VS Code"]  # 指定時はこれだけ監視
redaction_patterns = []        # LLM送信前にマスクする正規表現

[confidence]
decay_rate = 0.99              # パターンの信頼度減衰率（日次）
threshold_silent = 0.8         # これ以上なら質問しない
threshold_re_ask = 0.5         # 既存パターンの再確認閾値
threshold_soft_delete = 0.3    # パターンのソフトデリート閾値

[mascot]
character_image = "/path/to/character.png"  # キャラクター画像（透過PNG推奨、未指定時はデフォルト画像）
```

## データ保存場所

SQLiteデータベースが以下に保存されます:

- macOS: `~/Library/Application Support/shiritagari/shiritagari.db`
- Windows: `%APPDATA%\shiritagari\shiritagari.db`

## アーキテクチャ

```
src-tauri/src/
├── config.rs          # 設定ファイル読み込み
├── lib.rs             # Tauriアプリ統合・IPC・ポーリングループ
├── memory/            # 3層ハイブリッド記憶モデル（SQLite）
│   ├── patterns.rs    #   パターン記憶（長期）
│   ├── episodes.rs    #   エピソード記憶（1ヶ月で削除）
│   ├── speculations.rs#   推測ログ（3日で消滅）
│   ├── profile.rs     #   ユーザプロファイル
│   ├── confidence.rs  #   信頼度減衰・アクション判定
│   ├── cleanup.rs     #   定期クリーンアップ
│   └── promotion.rs   #   エピソード→パターン昇格
├── polling/           # ActivityWatch連携
│   ├── aw_client.rs   #   REST APIクライアント
│   ├── cursor.rs      #   カーソル永続化・冪等性
│   └── poller.rs      #   ポーリングループ
├── inference/         # 行動推論エンジン
│   └── engine.rs      #   パターン照合・LLM推論・質問判定
└── providers/         # LLMプロバイダ
    ├── types.rs       #   LlmProvider trait
    ├── claude.rs      #   Claude API
    ├── openai.rs      #   OpenAI API
    ├── ollama.rs      #   Ollama（ローカル）
    ├── factory.rs     #   プロバイダ生成
    └── redaction.rs   #   プライバシー保護（blocklist・マスキング）

src/                   # React フロントエンド
├── App.tsx            #   デスクトップマスコットUI
└── App.css            #   スタイル
```

## ライセンス

TBD
